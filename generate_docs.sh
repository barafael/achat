cargo doc --no-deps
rm -rf ./docs
echo "<meta http-equiv=\"refresh\" content=\"0; url=achat\">" > target/doc/index.html
cp -r target/doc ./docs

