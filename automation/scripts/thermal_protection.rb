# Thermal protection script
PIN = ENV.fetch('THERMAL_RELAY_PIN', 17).to_i
THRESHOLD = ENV.fetch('THERMAL_PROTECT_TEMP', 85).to_f

begin
  temp = Hardware.respond_to?(:cpu_temperature) ? Hardware.cpu_temperature : nil
  if temp && temp > THRESHOLD
    Hardware.write_gpio(PIN, 1)
  end
rescue => e
  STDERR.puts "thermal_protection error: #{e.class}: #{e.message}"
end
