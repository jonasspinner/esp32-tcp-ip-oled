use std::io::{self, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt};
use embedded_graphics::image::ImageRaw;
use embedded_graphics::pixelcolor::BinaryColor;
use rand::Rng;

#[derive(Clone)]
pub struct BitImage {
    bytes: Vec<u8>,
    height: usize,
    width: usize,
}

impl BitImage {
    pub fn new(width: usize, height: usize) -> Self {
        let n_bytes = height * width.div_ceil(8);
        let bytes = vec![0; n_bytes];
        Self { bytes, height, width }
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        let (byte_idx, bit_idx) = self.to_byte_and_bit_idx(x, y);
        if value {
            self.bytes[byte_idx] |= 1 << bit_idx;
        } else {
            self.bytes[byte_idx] &= !(1 << bit_idx);
        }
    }
    pub fn get(&self, x: usize, y: usize) -> bool {
        let (byte_idx, bit_idx) = self.to_byte_and_bit_idx(x, y);
        self.bytes[byte_idx] & (1 << bit_idx) != 0
    }
    fn to_byte_and_bit_idx(&self, x: usize, y: usize) -> (usize, usize) {
        assert!(x < self.width);
        assert!(y < self.height);
        let idx = x + y * self.width.next_multiple_of(8);
        (idx / 8, 7 - (idx % 8))
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn as_image_raw(&self) -> ImageRaw<BinaryColor> {
        ImageRaw::new(&self.bytes, self.width as u32)
    }
}


pub trait Serialize {
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize>;
}
pub trait Deserialize {
    type Output;
    fn deserialize(buf: &mut impl Read) -> io::Result<Self::Output>;
}


#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum CommandKind {
    Set0 = 1,
    Set1 = 2,
    StepGOL = 3,
    FillRandom = 4,
}

impl Serialize for CommandKind {
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize> {
        buf.write_u8(*self as u8)?;
        Ok(1)
    }
}
impl Deserialize for CommandKind {
    type Output = CommandKind;

    fn deserialize(buf: &mut impl Read) -> io::Result<Self::Output> {
        use CommandKind::*;
        let byte = buf.read_u8()?;
        match byte {
            1 => Ok(Set0),
            2 => Ok(Set1),
            3 => Ok(StepGOL),
            4 => Ok(FillRandom),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid CommandKind",
            ))
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Set0 { x: u8, y: u8 },
    Set1 { x: u8, y: u8 },
    StepGOL,
    FillRandom,
}

impl Command {
    fn kind(&self) -> CommandKind {
        use CommandKind::*;
        match self {
            Command::Set0 { .. } => Set0,
            Command::Set1 { .. } => Set1,
            Command::StepGOL => StepGOL,
            Command::FillRandom => FillRandom,
        }
    }
}

impl Serialize for Command {
    fn serialize(&self, buf: &mut impl Write) -> io::Result<usize> {
        let mut bytes_written = 0;
        bytes_written += self.kind().serialize(buf)?;
        match self {
            Command::Set0 { x, y } => {
                buf.write_u8(*x)?;
                buf.write_u8(*y)?;
                bytes_written += 2;
            }
            Command::Set1 { x, y } => {
                buf.write_u8(*x)?;
                buf.write_u8(*y)?;
                bytes_written += 2;
            }
            Command::StepGOL => {}
            Command::FillRandom => {}
        }
        Ok(bytes_written)
    }
}

impl Deserialize for Command {
    type Output = Command;

    fn deserialize(buf: &mut impl Read) -> io::Result<Self::Output> {
        use Command::*;
        let kind = CommandKind::deserialize(buf)?;
        match kind {
            CommandKind::Set0 => {
                let x = buf.read_u8()?;
                let y = buf.read_u8()?;
                Ok(Set0 { x, y })
            }
            CommandKind::Set1 => {
                let x = buf.read_u8()?;
                let y = buf.read_u8()?;
                Ok(Set1 { x, y })
            }
            CommandKind::StepGOL => Ok(StepGOL),
            CommandKind::FillRandom => Ok(FillRandom),
        }
    }
}


pub fn apply(image: &mut BitImage, command: Command, rng: &mut impl Rng) {
    match command {
        Command::Set0 { x, y } => { image.set(x as usize, y as usize, false) }
        Command::Set1 { x, y } => { image.set(x as usize, y as usize, true) }
        Command::StepGOL => { *image = step_game_of_life(&image); }
        Command::FillRandom => { fill_random(image, rng); }
    }
}


pub fn fill_random(image: &mut BitImage, rng: &mut impl Rng) {
    for y in 0..image.height() {
        for x in 0..image.width() {
            image.set(x, y, rng.gen());
        }
    }
}

pub fn step_game_of_life(current: &BitImage) -> BitImage {
    fn count_live_neighbors(image: &BitImage, x: usize, y: usize) -> usize {
        let x_size = image.width();
        let y_size = image.height();
        let x_deltas = [x_size - 1, 0, 1];
        let y_deltas = [y_size - 1, 0, 1];
        let mut count = 0;
        for x_delta in x_deltas {
            for y_delta in y_deltas {
                if x_delta == 0 && y_delta == 0 { continue; }
                let x_neighbor = (x + x_delta) % x_size;
                let y_neighbor = (y + y_delta) % y_size;
                if image.get(x_neighbor, y_neighbor) {
                    count += 1;
                }
            }
        }
        count
    }

    let mut next = current.clone();

    for y in 0..current.height() {
        for x in 0..current.width() {
            let alive = current.get(x, y);
            let count = count_live_neighbors(current, x, y);
            let next_alive = match (alive, count) {
                (true, 2 | 3) => true,
                (false, 3) => true,
                (_, _) => false,
            };
            next.set(x, y, next_alive);
        }
    }
    next
}

#[cfg(test)]
mod test {
    use crate::BitImage;

    #[test]
    fn bit_image() {
        let mut image = BitImage::new(1, 1);
        assert_eq!(image.as_bytes(), &[0b0000_0000]);
        assert_eq!(image.get(0, 0), false);
        image.set(0, 0, true);
        assert_eq!(image.as_bytes(), &[0b1000_0000]);
        assert_eq!(image.get(0, 0), true);

        let mut image = BitImage::new(3, 5);
        assert_eq!(image.as_bytes(), &[0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]);
        image.set(0, 0, true);
        image.set(1, 1, true);
        image.set(2, 2, true);
        image.set(1, 3, true);
        image.set(0, 4, true);
        assert_eq!(image.as_bytes(), &[
            0b1000_0000,
            0b0100_0000,
            0b0010_0000,
            0b0100_0000,
            0b1000_0000
        ]);

        let mut image = BitImage::new(9, 9);
        for i in 0..9 {
            image.set(i, i, true);
        }
        assert_eq!(image.as_bytes(), &[
            0b1000_0000, 0b0000_0000,
            0b0100_0000, 0b0000_0000,
            0b0010_0000, 0b0000_0000,
            0b0001_0000, 0b0000_0000,
            0b0000_1000, 0b0000_0000,
            0b0000_0100, 0b0000_0000,
            0b0000_0010, 0b0000_0000,
            0b0000_0001, 0b0000_0000,
            0b0000_0000, 0b1000_0000
        ]);
    }
}