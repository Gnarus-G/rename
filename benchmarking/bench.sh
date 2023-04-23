echo "--- perl-rename";
echo "setting up..."
mkdir files;
touch files/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
echo "running..."
time perl-rename "s/g-(\d+)-a-(\d+)-al-(\d+)/artist-\2-album-\3-genre-\1/" files/g*;
rm -r files;

echo;
echo "--- rename simple";
echo "setting up..."
mkdir files;
touch files/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
echo "running..."
time rn simple "g-(g:int)-a-(a:int)-al-(al:int)->artist-(a)-album-(al)-genre-(g)" files/g*;
rm -r files;
