PIN = ENV.fetch('FAN_GPIO_PIN', 17).to_i
THRESHOLD = ENV.fetch('FAN_ON_TEMP', 70).to_f
INTERVAL = ENV.fetch('FAN_CHECK_INTERVAL', 5).to_i

loop do
  begin
    temp = Hardware.respond_to?(:cpu_temperature) ? Hardware.cpu_temperature : nil
    if temp && temp > THRESHOLD
      Hardware.write_gpio(PIN, 1)
    else
      Hardware.write_gpio(PIN, 0)
    end
  rescue => e
    STDERR.puts "fan_control error: #{e.class}: #{e.message}"
  end
  sleep INTERVAL
end
