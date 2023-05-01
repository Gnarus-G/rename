cargo build --release;

echo "--- perl-rename";
echo "setting up ~54K files..."
mkdir files;
touch files/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
echo "running..."
time perl-rename "s/g-(\d+)-a-(\d+)-al-(\d+)/artist-\2-album-\3-genre-\1/" files/g*;
rm -r files;

echo;
echo "--- rename simple";
echo "setting up ~54K files..."
mkdir files;
touch files/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
echo "running..."
time ../target/release/rn simple "g-(g:int)-a-(a:int)-al-(al:int)->artist-(a)-album-(al)-genre-(g)" files/g*;
rm -r files;

echo;
echo "--- rename simple in parallel";
echo "setting up ~1.5M files..."
mkdir files;
mkdir files1;
mkdir files2;
mkdir files3;
mkdir files4;
mkdir files5;
mkdir files6;
mkdir files7;
mkdir files8;
mkdir files9;
mkdir files10;
mkdir files11;
mkdir files12;
mkdir files13;
mkdir files14;
mkdir files15;
mkdir files16;
mkdir files17;
mkdir files18;
mkdir files19;
mkdir files20;
mkdir files21;
mkdir files22;
mkdir files23;
mkdir files24;
mkdir files25;
mkdir files26;
mkdir files27;
touch files/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files1/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files2/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files3/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files4/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files5/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files6/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files7/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files8/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files9/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files10/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files11/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files12/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files13/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files14/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files15/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files16/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files17/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files18/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files19/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files20/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files21/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files22/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files23/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files24/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files25/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files26/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
touch files27/g-{0001..0038}-a-{0001..0038}-al-{0001..0038}; # ~54K files
echo "running..."
time ../target/release/rn simple "g-(g:int)-a-(a:int)-al-(al:int)->artist-(a)-album-(al)-genre-(g)" --multi --glob "files*/g*";
rm -r files;
rm -r files*;
