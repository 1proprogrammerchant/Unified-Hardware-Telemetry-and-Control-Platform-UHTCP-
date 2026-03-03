loop do
  if Hardware.cpu_temperature > 70
    Hardware.write_gpio(17, 1)
  else
    Hardware.write_gpio(17, 0)
  end
  sleep 5
end
