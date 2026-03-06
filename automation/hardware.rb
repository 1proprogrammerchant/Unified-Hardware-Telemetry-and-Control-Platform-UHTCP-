require 'logger'

module Hardware
  @logger = Logger.new($stderr)

  class << self
    # Return current CPU temperature in degrees Celsius (Float), or nil when
    # unavailable. In production this may proxy to the platform state.
    def cpu_temperature
      path = '/sys/class/thermal/thermal_zone0/temp'
      return nil unless File.exist?(path)
      raw = File.read(path).to_i
      raw / 1000.0
    rescue => e
      @logger.debug("cpu_temperature read failed: #{e.class}: #{e.message}")
      nil
    end

    # Write a GPIO pin. In the automation harness this should send an IPC
    # message to the controller; here we provide a safe stub that logs.
    def write_gpio(pin, value)
      # TODO: replace with IPC to server or platform-specific driver
      @logger.info("write_gpio pin=#{pin} value=#{value}")
    end
  end
end
