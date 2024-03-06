pid=$(lsof -i :8899 | awk '$1 == "COMMAND" {next} {print $2; exit}')

if [ -n "$pid" ]; then
    echo "Killing process with PID $pid"
    kill $pid
else
    echo "No process found for the specified port"
fi