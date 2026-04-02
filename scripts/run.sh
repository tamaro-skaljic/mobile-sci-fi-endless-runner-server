#/bin/sh

echo "Building module..."
echo
spacetime publish --server local mobile-sci-fi-endless-runner-server
echo

echo "Showing logs..."
echo
spacetime logs --server local mobile-sci-fi-endless-runner-server
echo

echo "Cleaning up module..."
echo
spacetime delete --server local mobile-sci-fi-endless-runner-server
echo