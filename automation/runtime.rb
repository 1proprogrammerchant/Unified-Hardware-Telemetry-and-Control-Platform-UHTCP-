require_relative 'hardware'
require_relative 'scheduler'

logger = Logger.new($stdout) rescue nil
logger&.info('automation runtime start')

# Example: initialize scheduler and keep process alive. The runtime here is a
# minimal shim — real startup may register scripts with an external service.
scheduler = AutomationScheduler.new

# Keep main thread alive to allow scheduled jobs to run.
trap('INT') { scheduler.shutdown; exit }
trap('TERM') { scheduler.shutdown; exit }

sleep while true
