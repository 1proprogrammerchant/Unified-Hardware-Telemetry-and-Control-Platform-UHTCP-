#!/usr/bin/env ruby
require 'json'
require 'thread'

# Simple sandbox runner: provides a `Hardware` API to user scripts.
# Communication protocol: runner reads JSON messages from STDIN (from supervisor)
# with shape {"type":"state","state":{...}} and sends actions to STDOUT
# like {"type":"action","action":"write_gpio","pin":17,"value":1}.

class Runner
  def initialize(script_path)
    @script_path = script_path
    @state = {}
    @state_mutex = Mutex.new
    @in_queue = Queue.new
    @out_mutex = Mutex.new
  end

  def start
    Thread.new { stdin_loop }
    Thread.new { run_script }
    # indicate ready
    send_msg(type: 'ready')
    sleep while true
  end

  def stdin_loop
    STDIN.each_line do |line|
      begin
        msg = JSON.parse(line)
        if msg['type'] == 'state' && msg['state']
          @state_mutex.synchronize { @state = msg['state'] }
        else
          @in_queue << msg
        end
      rescue => e
        # ignore malformed
      end
    end
  end

  def run_script
    code = File.read(@script_path)
    # Provide a minimal Hardware API
    runner = self
    Hardware = Module.new do
      extend self
    end

    def Hardware.read_state
      runner = Thread.current[:runner]
      runner.get_state
    end

    def Hardware.write_gpio(pin, value)
      runner = Thread.current[:runner]
      runner.send_action('write_gpio', pin: pin, value: value)
    end

    # Execute in safe thread-local context
    t = Thread.new do
      Thread.current[:runner] = self
      begin
        eval(code, binding, @script_path)
      rescue Exception => e
        send_msg(type: 'error', error: e.message)
      end
    end
    t.join
  end

  def get_state
    @state_mutex.synchronize { @state.dup }
  end

  def send_action(action, payload = {})
    send_msg(type: 'action', action: action, **payload)
  end

  def send_msg(msg)
    @out_mutex.synchronize do
      STDOUT.puts(msg.to_json)
      STDOUT.flush
    end
  end
end

if ARGV.length < 1
  STDERR.puts "usage: runner.rb path/to/script.rb"
  exit 2
end

runner = Runner.new(ARGV[0])
runner.start
