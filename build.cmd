cd frontend
trunk build --release --public-url dist
cd ..

cd backend
cargo build --release
cd ..