#!/bin/bash

echo "Programming SPI flash..."
if ! openocd -f "openocd/target_mario.cfg" -f "openocd/interface_cmsis-dap.cfg" \
    -c "init;" \
    -c "reset halt;" \
    -c "program ./game-and-watch-stm32/assets/crab_rave.raw_s16le_pcm_48k 0x90000000 verify;" \
    -c "exit;" 2>&1; then
    echo "Programming SPI flash failed. Check debug connection and try again."
    exit 1
fi

echo "Success"
echo "(You should power-cycle the device now)"
