# Example automation script: toggles GPIO pin 4 every second using the sandbox API
loop do
  Hardware.write_gpio(4, 1)
  sleep 1
  Hardware.write_gpio(4, 0)
  sleep 1
end
