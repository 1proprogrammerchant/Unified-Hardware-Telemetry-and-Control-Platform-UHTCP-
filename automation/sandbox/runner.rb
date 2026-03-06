#!/usr/bin/env ruby
require 'json'
require 'thread'
require 'logger'

# Sandbox Runner
# - Reads JSON messages from STDIN and exposes a minimal `Hardware` API
#   to user scripts executed in a dedicated thread. Outgoing actions are
#   written to STDOUT as JSON messages.

class Runner
  def initialize(script_path)
    @script_path = script_path
    @state = {}
    @state_mutex = Mutex.new
    @in_queue = Queue.new
    @out_mutex = Mutex.new
    @logger = Logger.new($stderr)
    @logger.progname = 'Runner'
    @running = true
  end

  def start
    @stdin_thread = Thread.new { stdin_loop }
    @script_thread = Thread.new { run_script }
    send_msg(type: 'ready')
    # main thread waits for children
    @script_thread.join
  ensure
    shutdown
  end

  def shutdown
    @running = false
    @stdin_thread.kill if @stdin_thread&.alive?
  rescue => e
    @logger.debug("shutdown error: #{e.message}")
  end

  def stdin_loop
    STDIN.each_line do |line|
      break unless @running
      begin
        msg = JSON.parse(line)
        if msg['type'] == 'state' && msg['state']
          @state_mutex.synchronize { @state = msg['state'] }
        else
          @in_queue << msg
        end
      rescue JSON::ParserError => _
        @logger.warn('dropping malformed JSON from stdin')
      rescue => e
        @logger.error("stdin loop error: #{e.class}: #{e.message}")
      end
    end
  end

  def run_script
    code = File.read(@script_path)

    # Provide a minimal Hardware API as a top-level constant for scripts.
    # We create methods that forward to the runner instance via closure.
    hardware_module = Module.new

    hardware_module.define_singleton_method(:read_state) do
      # runner instance will be passed via thread-local :runner
      Thread.current[:runner]&.get_state
    end

    hardware_module.define_singleton_method(:write_gpio) do |pin, value|
      Thread.current[:runner]&.send_action('write_gpio', pin: pin, value: value)
    end

    # Set the constant `Hardware` for the evaluated script. This mirrors the
    # sandbox API the automation scripts expect.
    Object.const_set(:Hardware, hardware_module)

    # Execute the script in its own thread and provide a reference to this
    # Runner instance via thread-local storage so the Hardware module can
    # forward calls back to the runner.
    t = Thread.new do
      Thread.current[:runner] = self
      begin
        # Use a clean binding to limit access to runner internals.
        eval(code, TOPLEVEL_BINDING, @script_path)
      rescue SystemExit
        # allow scripts to exit cleanly
      rescue Exception => e
        send_msg(type: 'error', error: e.message, backtrace: e.backtrace[0..5])
      end
    end
    t.join
  ensure
    # remove the Hardware constant to avoid leaking into other code
    Object.send(:remove_const, :Hardware) if Object.const_defined?(:Hardware)
  end

  def get_state
    @state_mutex.synchronize { @state.dup }
  end

  def send_action(action, payload = {})
    send_msg(type: 'action', action: action, payload: payload)
  end

  def send_msg(msg)
    @out_mutex.synchronize do
      STDOUT.puts(msg.to_json)
      STDOUT.flush
    end
  rescue => e
    @logger.error("failed to send msg: #{e.message}")
  end
end

if ARGV.length < 1
  STDERR.puts 'usage: runner.rb path/to/script.rb'
  exit 2
end

runner = Runner.new(ARGV[0])
runner.start
