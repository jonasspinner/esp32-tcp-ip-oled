# ESP32 OLED via TCP

#### Enable USB

    sudo chmod a+rw /dev/ttyUSB0


## ESP32

    env WIFI_SSID=... WIFI_PASS=... cargo run


## Client

    cd client && cargo run