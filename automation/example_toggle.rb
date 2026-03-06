PIN = ENV.fetch('TOGGLE_GPIO_PIN', 4).to_i
INTERVAL = ENV.fetch('TOGGLE_INTERVAL', 1).to_f

begin
  loop do
    Hardware.write_gpio(PIN, 1)
    sleep INTERVAL
    Hardware.write_gpio(PIN, 0)
    sleep INTERVAL
  end
rescue Interrupt
  # allow graceful termination
rescue => e
  STDERR.puts "example_toggle error: #{e.class}: #{e.message}"
end
