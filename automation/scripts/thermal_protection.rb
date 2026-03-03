# Thermal protection
if Hardware.cpu_temperature > 85
  Hardware.write_gpio(17, 1)
end
