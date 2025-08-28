#!/bin/bash

# Thresh Wallet Logging Script

LOG_FILE="/tmp/thresh.log"

case "${1:-tail}" in
  "tail"|"follow"|"f")
    echo "Following logs in real-time (Ctrl+C to stop)..."
    echo "Log file: $LOG_FILE"
    echo "----------------------------------------"
    tail -f "$LOG_FILE" 2>/dev/null || echo "No log file found. Try connecting to the extension first."
    ;;
    
  "show"|"cat")
    echo "Recent log entries:"
    echo "----------------------------------------"
    if [ -f "$LOG_FILE" ]; then
      tail -50 "$LOG_FILE"
    else
      echo "No log file found. Try connecting to the extension first."
    fi
    ;;
    
  "clear"|"clean")
    if [ -f "$LOG_FILE" ]; then
      rm "$LOG_FILE"
      echo "Log file cleared."
    else
      echo "No log file to clear."
    fi
    ;;
    
  "help"|"-h"|"--help")
    echo "Thresh Wallet Logging Script"
    echo ""
    echo "Usage: ./logs.sh [command]"
    echo ""
    echo "Commands:"
    echo "  tail, follow, f    Follow logs in real-time (default)"
    echo "  show, cat          Show recent log entries"
    echo "  clear, clean       Clear the log file"
    echo "  help, -h, --help   Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./logs.sh          # Follow logs in real-time"
    echo "  ./logs.sh show     # Show recent entries"
    echo "  ./logs.sh clear    # Clear logs"
    ;;
    
  *)
    echo "Unknown command: $1"
    echo "Use './logs.sh help' for usage information."
    exit 1
    ;;
esac