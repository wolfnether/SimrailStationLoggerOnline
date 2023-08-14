cd frontend
trunk build --release --public-url dist --filehash false
cd ..

cd backend
cargo build --release
cd ..