object #93
  name: "Conversion Utils"
  parent: #78
  location: #-1
  owner: #36
  readable: true
  override "key" = 0;

  override "aliases" = {"Conversion Utils"};

  override "description" = "This is a utilities package for converting from one unit of measurement to another. Type 'help #770' for more details.";

  override "help_msg" = {"Utility verbs for converting from one unit of measure to another.", "", "Unusual conversions:", ":dd_to_dms => converts decimal (INT or FLOAT) Degrees into Degrees, Minutes,", "              and Seconds. (Also works for decimal Hours.)", ":dms_to_dd => converts from Degrees (or Hours), Minutes, and Seconds to", "              decimal Degrees (or Hours).", ":rect_to_polar => converts from cartesian (x,y) coordinates to polar.", ":polar_to_rect => converts from polar (r, theta) coordinates to cartesian.", ":F_to_C => converts from Fahrenheit to Celsius.", ":C_to_F => converts from Celsius to Fahrenheit.", ":C_to_K => converts from Celsius to Kelvin.", ":K_to_C => converts from Kelvin to Celsius.", ":F_to_R => converts from Fahrenheit to Rankine.", ":R_to_F => converts from Rankine to Fahrenheit.", "", "Standard conversions:", ":convert => takes two string inputs and attempts to determine the ", "            multiplicative conversion factor. See the verb help for details", "            and input format.\""};

  property "basic_units" (owner: #36, flags: "r") = {"m", "kg", "s", "coul", "candela", "radian", "bit", "erlang", "kelvin"};

  property "pi" (owner: #36, flags: "r") = "3.14159265358979323846264338327950288";

  property "c" (owner: #36, flags: "r") = "2.99792458e8 m/sec";

  property "g" (owner: #36, flags: "r") = "9.80665 m/sec2";

  property "au" (owner: #36, flags: "r") = "1.49599e11 m";

  property "mole" (owner: #36, flags: "r") = "6.022045e23";

  property "e" (owner: #36, flags: "r") = "1.6020e-19 coul";

  property "abcoulomb" (owner: #36, flags: "r") = "10 coul";

  property "force" (owner: #36, flags: "r") = "g";

  property "slug" (owner: #36, flags: "r") = "lb g sec2/ft";

  property "mercury" (owner: #36, flags: "r") = "1.3157895 atm/m";

  property "hg" (owner: #36, flags: "r") = "mercury";

  property "torr" (owner: #36, flags: "r") = "mm hg";

  property "%" (owner: #36, flags: "r") = "1|100";

  property "percent" (owner: #36, flags: "r") = "%";

  property "cg" (owner: #36, flags: "r") = "centigram";

  property "atmosphere" (owner: #36, flags: "r") = "1.01325 bar";

  property "atm" (owner: #36, flags: "r") = "atmosphere";

  property "psi" (owner: #36, flags: "r") = "lb g/in2";

  property "bar" (owner: #36, flags: "r") = "1e6 dyne/cm2";

  property "chemamu" (owner: #36, flags: "r") = "1.66024e-24 g";

  property "physamu" (owner: #36, flags: "r") = "1.65979e-24 g";

  property "amu" (owner: #36, flags: "r") = "chemamu";

  property "chemdalton" (owner: #36, flags: "r") = "chemamu";

  property "dalton" (owner: #36, flags: "r") = "chemamu";

  property "physdalton" (owner: #36, flags: "r") = "physamu";

  property "dozen" (owner: #36, flags: "r") = "12";

  property "bakersdozen" (owner: #36, flags: "r") = "13";

  property "quire" (owner: #36, flags: "r") = "25";

  property "ream" (owner: #36, flags: "r") = "500";

  property "gross" (owner: #36, flags: "r") = "144";

  property "hertz" (owner: #36, flags: "r") = "1/sec";

  property "cps" (owner: #36, flags: "r") = "hertz";

  property "hz" (owner: #36, flags: "r") = "hertz";

  property "khz" (owner: #36, flags: "r") = "kilohz";

  property "mhz" (owner: #36, flags: "r") = "megahz";

  property "rutherford" (owner: #36, flags: "r") = "1e6/sec";

  property "degree" (owner: #36, flags: "r") = "1|180 pi radian";

  property "circle" (owner: #36, flags: "r") = "2 pi radian";

  property "turn" (owner: #36, flags: "r") = "2 pi radian";

  property "revolution" (owner: #36, flags: "r") = "360 degrees";

  property "rpm" (owner: #36, flags: "r") = "revolution/minute";

  property "grade" (owner: #36, flags: "r") = "1|400 circle";

  property "grad" (owner: #36, flags: "r") = "1|400 circle";

  property "sign" (owner: #36, flags: "r") = "1|12 circle";

  property "arcdeg" (owner: #36, flags: "r") = "1 degree";

  property "arcmin" (owner: #36, flags: "r") = "1|60 arcdeg";

  property "arcsec" (owner: #36, flags: "r") = "1|60 arcmin";

  property "karat" (owner: #36, flags: "r") = "1|24";

  property "proof" (owner: #36, flags: "r") = "1|200";

  property "mpg" (owner: #36, flags: "r") = "mile/gal";

  property "curie" (owner: #36, flags: "r") = "3.7e10/sec";

  property "stoke" (owner: #36, flags: "r") = "1 cm2/sec";

  property "steradian" (owner: #36, flags: "r") = "radian radian";

  property "sr" (owner: #36, flags: "r") = "steradian";

  property "sphere" (owner: #36, flags: "r") = "4 pi steradian";

  property "ps" (owner: #36, flags: "r") = "picosec";

  property "us" (owner: #36, flags: "r") = "microsec";

  property "ns" (owner: #36, flags: "r") = "nanosec";

  property "ms" (owner: #36, flags: "r") = "millisec";

  property "sec" (owner: #36, flags: "r") = "second";

  property "minute" (owner: #36, flags: "r") = "60 sec";

  property "min" (owner: #36, flags: "r") = "minute";

  property "hour" (owner: #36, flags: "r") = "60 min";

  property "hr" (owner: #36, flags: "r") = "hour";

  property "day" (owner: #36, flags: "r") = "24 hr";

  property "week" (owner: #36, flags: "r") = "7 day";

  property "quadrant" (owner: #36, flags: "r") = "5400 minute";

  property "fortnight" (owner: #36, flags: "r") = "14 day";

  property "year" (owner: #36, flags: "r") = "365.24219879 day";

  property "yr" (owner: #36, flags: "r") = "year";

  property "month" (owner: #36, flags: "r") = "1|12 year";

  property "mo" (owner: #36, flags: "r") = "month";

  property "decade" (owner: #36, flags: "r") = "10 year";

  property "century" (owner: #36, flags: "r") = "100 year";

  property "millenium" (owner: #36, flags: "r") = "1000 year";

  property "gm" (owner: #36, flags: "r") = "gram";

  property "myriagram" (owner: #36, flags: "r") = "10 kg";

  property "mg" (owner: #36, flags: "r") = "milligram";

  property "metricton" (owner: #36, flags: "r") = "1000 kg";

  property "gamma" (owner: #36, flags: "r") = "1e-6 g";

  property "metriccarat" (owner: #36, flags: "r") = "200 mg";

  property "quintal" (owner: #36, flags: "r") = "100 kg";

  property "lb" (owner: #36, flags: "r") = "0.45359237 kg";

  property "pound" (owner: #36, flags: "r") = "lb";

  property "lbf" (owner: #36, flags: "r") = "lb g";

  property "cental" (owner: #36, flags: "r") = "100 lb";

  property "stone" (owner: #36, flags: "r") = "14 lb";

  property "ounce" (owner: #36, flags: "r") = "1|16 lb";

  property "oz" (owner: #36, flags: "r") = "ounce";

  property "avdram" (owner: #36, flags: "r") = "1|16 oz";

  property "usdram" (owner: #36, flags: "r") = "1|8 oz";

  property "dram" (owner: #36, flags: "r") = "avdram";

  property "dr" (owner: #36, flags: "r") = "dram";

  property "grain" (owner: #36, flags: "r") = "1|7000 lb";

  property "gr" (owner: #36, flags: "r") = "grain";

  property "shortton" (owner: #36, flags: "r") = "2000 lb";

  property "ton" (owner: #36, flags: "r") = "shortton";

  property "longquarter" (owner: #36, flags: "r") = "28 lb";

  property "shortquarter" (owner: #36, flags: "r") = "500 lb";

  property "longton" (owner: #36, flags: "r") = "2240 lb";

  property "longhundredweight" (owner: #36, flags: "r") = "112 lb";

  property "shorthundredweight" (owner: #36, flags: "r") = "100 lb";

  property "wey" (owner: #36, flags: "r") = "252 lb";

  property "carat" (owner: #36, flags: "r") = "205.3 mg";

  property "scruple" (owner: #36, flags: "r") = "20 grain";

  property "pennyweight" (owner: #36, flags: "r") = "24 grain";

  property "apdram" (owner: #36, flags: "r") = "60 grain";

  property "apounce" (owner: #36, flags: "r") = "480 grain";

  property "appound" (owner: #36, flags: "r") = "5760 grain";

  property "cm" (owner: #36, flags: "r") = "centimeter";

  property "mm" (owner: #36, flags: "r") = "millimeter";

  property "km" (owner: #36, flags: "r") = "kilometer";

  property "parsec" (owner: #36, flags: "r") = "au radian/arcsec";

  property "pc" (owner: #36, flags: "r") = "parsec";

  property "nm" (owner: #36, flags: "r") = "nanometer";

  property "micron" (owner: #36, flags: "r") = "1e-6 meter";

  property "angstrom" (owner: #36, flags: "r") = "1e-8 meter";

  property "fermi" (owner: #36, flags: "r") = "1e-13 cm";

  property "point" (owner: #36, flags: "r") = "1|72.27 in";

  property "pica" (owner: #36, flags: "r") = "0.166044 inch";

  property "caliber" (owner: #36, flags: "r") = "0.01 in";

  property "barleycorn" (owner: #36, flags: "r") = "1|3 in";

  property "inch" (owner: #36, flags: "r") = "2.54 cm";

  property "in" (owner: #36, flags: "r") = "inch";

  property "mil" (owner: #36, flags: "r") = "0.001 in";

  property "palm" (owner: #36, flags: "r") = "3 in";

  property "hand" (owner: #36, flags: "r") = "4 in";

  property "span" (owner: #36, flags: "r") = "9 in";

  property "foot" (owner: #36, flags: "r") = "12 in";

  property "feet" (owner: #36, flags: "r") = "foot";

  property "ft" (owner: #36, flags: "r") = "foot";

  property "cubit" (owner: #36, flags: "r") = "18 in";

  property "pace" (owner: #36, flags: "r") = "30 inch";

  property "yard" (owner: #36, flags: "r") = "3 ft";

  property "yd" (owner: #36, flags: "r") = "yard";

  property "fathom" (owner: #36, flags: "r") = "6 ft";

  property "rod" (owner: #36, flags: "r") = "16.5 ft";

  property "rd" (owner: #36, flags: "r") = "rod";

  property "rope" (owner: #36, flags: "r") = "20 ft";

  property "ell" (owner: #36, flags: "r") = "45 in";

  property "skein" (owner: #36, flags: "r") = "360 feet";

  property "cable" (owner: #36, flags: "r") = "720 ft";

  property "furlong" (owner: #36, flags: "r") = "660 ft";

  property "nmile" (owner: #36, flags: "r") = "1852 m";

  property "nautmile" (owner: #36, flags: "r") = "nmile";

  property "bolt" (owner: #36, flags: "r") = "120 feet";

  property "mile" (owner: #36, flags: "r") = "5280 feet";

  property "mi" (owner: #36, flags: "r") = "mile";

  property "league" (owner: #36, flags: "r") = "3 mi";

  property "nautleague" (owner: #36, flags: "r") = "3 nmile";

  property "lightyear" (owner: #36, flags: "r") = "c yr";

  property "engineerschain" (owner: #36, flags: "r") = "100 ft";

  property "engineerslink" (owner: #36, flags: "r") = "0.01 engineerschain";

  property "gunterchain" (owner: #36, flags: "r") = "66 ft";

  property "gunterlink" (owner: #36, flags: "r") = "0.01 gunterchain";

  property "ramdenchain" (owner: #36, flags: "r") = "100 ft";

  property "ramdenlink" (owner: #36, flags: "r") = "0.01 ramdenchain";

  property "acre" (owner: #36, flags: "r") = "43560 ft2";

  property "rood" (owner: #36, flags: "r") = "0.25 acre";

  property "are" (owner: #36, flags: "r") = "100 m2";

  property "centare" (owner: #36, flags: "r") = "0.01 are";

  property "hectare" (owner: #36, flags: "r") = "100 are";

  property "barn" (owner: #36, flags: "r") = "1e-24 cm2";

  property "section" (owner: #36, flags: "r") = "mi2";

  property "township" (owner: #36, flags: "r") = "36 mi2";

  property "cc" (owner: #36, flags: "r") = "cm3";

  property "liter" (owner: #36, flags: "r") = "1000 cc";

  property "l" (owner: #36, flags: "r") = "liter";

  property "ml" (owner: #36, flags: "r") = "milliliter";

  property "registerton" (owner: #36, flags: "r") = "100 ft3";

  property "cord" (owner: #36, flags: "r") = "128 ft3";

  property "boardfoot" (owner: #36, flags: "r") = "144 in3";

  property "boardfeet" (owner: #36, flags: "r") = "boardfoot";

  property "cordfoot" (owner: #36, flags: "r") = "0.125 cord";

  property "cordfeet" (owner: #36, flags: "r") = "cordfoot";

  property "last" (owner: #36, flags: "r") = "80 bu";

  property "perch" (owner: #36, flags: "r") = "24.75 ft3";

  property "stere" (owner: #36, flags: "r") = "m3";

  property "cfs" (owner: #36, flags: "r") = "ft3/sec";

  property "gallon" (owner: #36, flags: "r") = "231 in3";

  property "imperial" (owner: #36, flags: "r") = "1.200949";

  property "gal" (owner: #36, flags: "r") = "gallon";

  property "quart" (owner: #36, flags: "r") = "1|4 gal";

  property "qt" (owner: #36, flags: "r") = "quart";

  property "magnum" (owner: #36, flags: "r") = "2 qt";

  property "pint" (owner: #36, flags: "r") = "1|2 qt";

  property "pt" (owner: #36, flags: "r") = "pint";

  property "cup" (owner: #36, flags: "r") = "1|2 pt";

  property "gill" (owner: #36, flags: "r") = "1|4 pt";

  property "fifth" (owner: #36, flags: "r") = "1|5 gal";

  property "firkin" (owner: #36, flags: "r") = "72 pint";

  property "barrel" (owner: #36, flags: "r") = "31.5 gal";

  property "petrbarrel" (owner: #36, flags: "r") = "42 gal";

  property "hogshead" (owner: #36, flags: "r") = "63 gal";

  property "hd" (owner: #36, flags: "r") = "hogshead";

  property "tun" (owner: #36, flags: "r") = "252 gal";

  property "kilderkin" (owner: #36, flags: "r") = "18 imperial gal";

  property "noggin" (owner: #36, flags: "r") = "1 imperial gill";

  property "floz" (owner: #36, flags: "r") = "1|4 gill";

  property "fldr" (owner: #36, flags: "r") = "1|32 gill";

  property "tablespoon" (owner: #36, flags: "r") = "4 fldr";

  property "teaspoon" (owner: #36, flags: "r") = "1|3 tablespoon";

  property "minim" (owner: #36, flags: "r") = "1|480 floz";

  property "pk" (owner: #36, flags: "r") = "peck";

  property "bushel" (owner: #36, flags: "r") = "8 dry gal";

  property "dry" (owner: #36, flags: "r") = "268.8025 in3/gallon";

  property "bu" (owner: #36, flags: "r") = "bushel";

  property "british" (owner: #36, flags: "r") = "277.4193|231";

  property "brbucket" (owner: #36, flags: "r") = "4 dry british gal";

  property "brpeck" (owner: #36, flags: "r") = "2 dry british gal";

  property "brbushel" (owner: #36, flags: "r") = "8 dry british gal";

  property "brfirkin" (owner: #36, flags: "r") = "1.125 brbushel";

  property "dryquartern" (owner: #36, flags: "r") = "2.272980 l";

  property "liqquarten" (owner: #36, flags: "r") = "0.1420613 l";

  property "butt" (owner: #36, flags: "r") = "126 gal";

  property "bag" (owner: #36, flags: "r") = "3 brbushels";

  property "brbarrel" (owner: #36, flags: "r") = "4.5 brbushels";

  property "seam" (owner: #36, flags: "r") = "8 brbushels";

  property "drachm" (owner: #36, flags: "r") = "3.551531 ml";

  property "newton" (owner: #36, flags: "r") = "kg m/sec2";

  property "pascal" (owner: #36, flags: "r") = "nt/m2";

  property "nt" (owner: #36, flags: "r") = "newton";

  property "joule" (owner: #36, flags: "r") = "nt m";

  property "cal" (owner: #36, flags: "r") = "4.1868 joule";

  property "gramcalorie" (owner: #36, flags: "r") = "cal";

  property "calorie" (owner: #36, flags: "r") = "cal";

  property "btu" (owner: #36, flags: "r") = "1054.35 joule";

  property "frigorie" (owner: #36, flags: "r") = "kilocal";

  property "kcal" (owner: #36, flags: "r") = "kilocal";

  property "kcalorie" (owner: #36, flags: "r") = "kilocal";

  property "langley" (owner: #36, flags: "r") = "cal/cm cm";

  property "dyne" (owner: #36, flags: "r") = "erg/cm";

  property "poundal" (owner: #36, flags: "r") = "ft lb/sec2";

  property "pdl" (owner: #36, flags: "r") = "poundal";

  property "erg" (owner: #36, flags: "r") = "1e-7 joule";

  property "horsepower" (owner: #36, flags: "r") = "550 ft lb g/sec";

  property "hp" (owner: #36, flags: "r") = "horsepower";

  property "poise" (owner: #36, flags: "r") = "gram/cm sec";

  property "reyn" (owner: #36, flags: "r") = "6.89476e-6 centipoise";

  property "rhe" (owner: #36, flags: "r") = "1/poise";

  property "coul" (owner: #36, flags: "r") = "coulomb";

  property "statcoul" (owner: #36, flags: "r") = "3.335635e-10 coul";

  property "ampere" (owner: #36, flags: "r") = "coul/sec";

  property "abampere" (owner: #36, flags: "r") = "10 amp";

  property "amp" (owner: #36, flags: "r") = "ampere";

  property "watt" (owner: #36, flags: "r") = "joule/sec";

  property "volt" (owner: #36, flags: "r") = "watt/amp";

  property "v" (owner: #36, flags: "r") = "volt";

  property "abvolt" (owner: #36, flags: "r") = "10 volt";

  property "statvolt" (owner: #36, flags: "r") = "299.7930 volt";

  property "ohm" (owner: #36, flags: "r") = "volt/amp";

  property "abohm" (owner: #36, flags: "r") = "10 ohm";

  property "mho" (owner: #36, flags: "r") = "1/ohm";

  property "abmho" (owner: #36, flags: "r") = "10 mho";

  property "siemens" (owner: #36, flags: "r") = "mho";

  property "farad" (owner: #36, flags: "r") = "coul/volt";

  property "abfarad" (owner: #36, flags: "r") = "10 farad";

  property "statfarad" (owner: #36, flags: "r") = "1.112646e-12 farad";

  property "pf" (owner: #36, flags: "r") = "picofarad";

  property "abhenry" (owner: #36, flags: "r") = "10 henry";

  property "henry" (owner: #36, flags: "r") = "sec2/farad";

  property "stathenry" (owner: #36, flags: "r") = "8.987584e11 henry";

  property "mh" (owner: #36, flags: "r") = "millihenry";

  property "weber" (owner: #36, flags: "r") = "volt sec";

  property "gauss" (owner: #36, flags: "r") = "maxwell/cm2";

  property "electronvolt" (owner: #36, flags: "r") = "e volt";

  property "ev" (owner: #36, flags: "r") = "e volt";

  property "kev" (owner: #36, flags: "r") = "1e3 ev";

  property "mev" (owner: #36, flags: "r") = "1e6 ev";

  property "bev" (owner: #36, flags: "r") = "1e9 ev";

  property "faraday" (owner: #36, flags: "r") = "9.648456e4coul";

  property "gilbert" (owner: #36, flags: "r") = "0.7957747154 amp";

  property "oersted" (owner: #36, flags: "r") = "1 gilbert / cm";

  property "oe" (owner: #36, flags: "r") = "oersted";

  property "cd" (owner: #36, flags: "r") = "candela";

  property "lumen" (owner: #36, flags: "r") = "cd sr";

  property "lux" (owner: #36, flags: "r") = "lumen/m2";

  property "footcandle" (owner: #36, flags: "r") = "lumen/ft2";

  property "footlambert" (owner: #36, flags: "r") = "cd/pi ft2";

  property "lambert" (owner: #36, flags: "r") = "cd/pi cm2";

  property "phot" (owner: #36, flags: "r") = "lumen/cm2";

  property "stilb" (owner: #36, flags: "r") = "cd/cm2";

  property "candle" (owner: #36, flags: "r") = "cd";

  property "engcandle" (owner: #36, flags: "r") = "1.04 cd";

  property "germancandle" (owner: #36, flags: "r") = "1.05 cd";

  property "carcel" (owner: #36, flags: "r") = "9.61 cd";

  property "hefnerunit" (owner: #36, flags: "r") = ".92 cd";

  property "candlepower" (owner: #36, flags: "r") = "12.566370 lumen";

  property "baud" (owner: #36, flags: "r") = "bit/sec";

  property "byte" (owner: #36, flags: "r") = "8 bit";

  property "kb" (owner: #36, flags: "r") = "1024 byte";

  property "mb" (owner: #36, flags: "r") = "1024 kb";

  property "gb" (owner: #36, flags: "r") = "1024 mb";

  property "word" (owner: #36, flags: "r") = "4 byte";

  property "long" (owner: #36, flags: "r") = "4 word";

  property "block" (owner: #36, flags: "r") = "512 byte";

  property "mph" (owner: #36, flags: "r") = "mile/hr";

  property "knot" (owner: #36, flags: "r") = "nmile/hr";

  property "brknot" (owner: #36, flags: "r") = "6080 ft/hr";

  property "mach" (owner: #36, flags: "r") = "331.45 m/sec";

  property "energy" (owner: #36, flags: "r") = "c2";

  property "ccs" (owner: #36, flags: "r") = "1|36 erlang";

  property "peck" (owner: #36, flags: "r") = "2 dry gallon";

  property "arpentcan" (owner: #36, flags: "r") = "27.52 mi";

  property "apostilb" (owner: #36, flags: "r") = "cd/pi m2";

  property "arpentlin" (owner: #36, flags: "r") = "191.835 ft";

  property "atomicmassunit" (owner: #36, flags: "r") = "amu";

  property "barie" (owner: #36, flags: "r") = "1e-1 nt/m2";

  property "barye" (owner: #36, flags: "r") = "1e-1 nt/m2";

  property "biot" (owner: #36, flags: "r") = "10 amp";

  property "blondel" (owner: #36, flags: "r") = "cd/pi m2";

  property "bottommeasure" (owner: #36, flags: "r") = "1|40 in";

  property "refrigeration" (owner: #36, flags: "r") = "12000 but/ton hr";

  property "centesimalminute" (owner: #36, flags: "r") = "1e-2 grade";

  property "centesimalsecond" (owner: #36, flags: "r") = "1e-4 grade";

  property "chain" (owner: #36, flags: "r") = "gunterchain";

  property "circularinch" (owner: #36, flags: "r") = "1|4 pi in2";

  property "circularmil" (owner: #36, flags: "r") = "1e-6|4 pi in2";

  property "clusec" (owner: #36, flags: "r") = "1e-8 mm hg m3/s";

  property "coomb" (owner: #36, flags: "r") = "4 bu";

  property "crith" (owner: #36, flags: "r") = "9.06e-2 gram";

  property "dioptre" (owner: #36, flags: "r") = "1/m";

  property "displacementton" (owner: #36, flags: "r") = "35 ft3";

  property "dopplezentner" (owner: #36, flags: "r") = "100 kg";

  property "equivalentfootcandle" (owner: #36, flags: "r") = "lumen/pi ft2";

  property "equivalentlux" (owner: #36, flags: "r") = "lumen/pi m2";

  property "equivalentphot" (owner: #36, flags: "r") = "cd/pi cm2";

  property "finger" (owner: #36, flags: "r") = "7|8 in";

  property "franklin" (owner: #36, flags: "r") = "3.33564e-10 coul";

  property "galileo" (owner: #36, flags: "r") = "1e-2 m/sec2";

  property "geographicalmile" (owner: #36, flags: "r") = "nmile";

  property "hefnercandle" (owner: #36, flags: "r") = "hefnerunit";

  property "homestead" (owner: #36, flags: "r") = "1|4 mi2";

  property "hyl" (owner: #36, flags: "r") = "gram force sec2/m";

  property "imaginarycubicfoot" (owner: #36, flags: "r") = "1.4 ft3";

  property "jeroboam" (owner: #36, flags: "r") = "4|5 gal";

  property "line" (owner: #36, flags: "r") = "1|12 in";

  property "link" (owner: #36, flags: "r") = "66|100 ft";

  property "lusec" (owner: #36, flags: "r") = "1e-6 mm hg m3/s";

  property "marineleague" (owner: #36, flags: "r") = "3nmile";

  property "maxwell" (owner: #36, flags: "r") = "1e-8 weber";

  property "mgd" (owner: #36, flags: "r") = "megagal/day";

  property "minersinch" (owner: #36, flags: "r") = "1.5 ft3/min";

  property "nail" (owner: #36, flags: "r") = "1|16 yd";

  property "nit" (owner: #36, flags: "r") = "cd/m2";

  property "nox" (owner: #36, flags: "r") = "1e-3 lux";

  property "pieze" (owner: #36, flags: "r") = "1e3 nt/mt2";

  property "pipe" (owner: #36, flags: "r") = "4 barrel";

  property "pole" (owner: #36, flags: "r") = "rd";

  property "quarter" (owner: #36, flags: "r") = "9 in";

  property "quartersection" (owner: #36, flags: "r") = "1|4 mi2";

  property "ra" (owner: #36, flags: "r") = "100 erg/gram";

  property "rankine" (owner: #36, flags: "r") = "1.8 kelvin";

  property "rehoboam" (owner: #36, flags: "r") = "156 floz";

  property "rontgen" (owner: #36, flags: "r") = "2.58e-4 curie/kg";

  property "rydberg" (owner: #36, flags: "r") = "1.36054e1 ev";

  property "sabin" (owner: #36, flags: "r") = "1 ft2";

  property "shippington" (owner: #36, flags: "r") = "40 ft3";

  property "sigma" (owner: #36, flags: "r") = "microsec";

  property "skot" (owner: #36, flags: "r") = "1e-3 apostilb";

  property "spat" (owner: #36, flags: "r") = "sphere";

  property "spindle" (owner: #36, flags: "r") = "14400 yd";

  property "square" (owner: #36, flags: "r") = "100 ft2";

  property "sthene" (owner: #36, flags: "r") = "1e3 nt";

  property "tesla" (owner: #36, flags: "r") = "weber/m2";

  property "thermie" (owner: #36, flags: "r") = "1e6 cal";

  property "timberfoot" (owner: #36, flags: "r") = "ft3";

  property "tonne" (owner: #36, flags: "r") = "1e6 gram";

  property "water" (owner: #36, flags: "r") = "0.22491|2.54 kg/m2 sec2";

  property "xunit" (owner: #36, flags: "r") = "1.00202e-13 m";

  property "k" (owner: #36, flags: "r") = "1.38047e-16 erg/kelvin";

  property "puncheon" (owner: #36, flags: "r") = "84 gal";

  property "tnt" (owner: #36, flags: "r") = "4.6e6 m2/sec2";

  property "basic_units_template" (owner: #36, flags: "r") = {{"m", 0}, {"kg", 0}, {"s", 0}, {"coul", 0}, {"candela", 0}, {"radian", 0}, {"bit", 0}, {"erlang", 0}, {"kelvin", 0}};

  property "meter" (owner: #36, flags: "r") = "m";

  property "gram" (owner: #36, flags: "r") = "1|1000 kg";

  property "second" (owner: #36, flags: "r") = "s";

  property "inches" (owner: #36, flags: "r") = "inch";

  property "sennight" (owner: #36, flags: "r") = "1 week";

  property "cubichectare" (owner: #36, flags: "r") = "1000000 m3";

  property "astronomicalunit" (owner: #2, flags: "r") = "au";

  property "fluidounce" (owner: #2, flags: "r") = "floz";

  property "tsp" (owner: #2, flags: "r") = "teaspoon";

  property "tbsp" (owner: #2, flags: "r") = "tablespoon";

  verb "dd_to_dms dh_to_hms" (this none this) owner: #36 flags: "rxd"
    ":dd_to_dms(INT|FLOAT <degrees>) => LIST {INT <degrees>, INT <minutes>, FLOAT <seconds>}";
    "This verb converts decimal degrees to degrees, minutes, and seconds.";
    dd = tofloat(args[1]);
    s = ((dd - tofloat(d = toint(dd))) * 60.0 - tofloat(m = toint((dd - tofloat(d)) * 60.0))) * 60.0;
    return {d, m, s};
  endverb

  verb "dms_to_dd hms_to_dh" (this none this) owner: #36 flags: "rxd"
    ":dms_to_dd(INT|FLOAT <deg>, INT|FLOAT <min>, INT|FLOAT <sec>) => FLOAT <deg>";
    "This verb converts degrees/minutes/seconds to decimal degrees.";
    {d, m, s} = args[1..3];
    d = tofloat(d);
    m = tofloat(m);
    s = tofloat(s);
    return d + m / 60.0 + s / 3600.0;
  endverb

  verb "rect_to_polar" (this none this) owner: #36 flags: "rxd"
    ":rect_to_polar(INT|FLOAT <x>, INT|FLOAT <y>) => FLOAT <radius>, FLOAT <angle>.";
    "This verb converts from rectangular (x,y) coordinates to polar (r, theta) coordinates.";
    {x, y} = args[1..2];
    x = tofloat(x);
    y = tofloat(y);
    return {sqrt(x * x + x * x), `atan(y, x) ! E_INVARG => 0.0'};
  endverb

  verb "polar_to_rect" (this none this) owner: #36 flags: "rxd"
    ":polar_to_rect(INT|FLOAT <radius>, INT|FLOAT <angle>) => FLOAT <x>, FLOAT <y>";
    "This verb converts from polar (radius, angle) coordinates to rectangulat (x,y) coordinates.";
    {r, a} = args[1..2];
    r = tofloat(r);
    a = tofloat(a);
    return {(r = r / (1.0 + (z2 = (z = tan(a / 2.0)) * z))) * (1.0 - z2), r * 2.0 * z};
  endverb

  verb "F_to_C degF_to_degC" (this none this) owner: #36 flags: "rxd"
    ":F_to_C(INT|FLOAT <Fahrenheit>) => FLOAT <Celsius>";
    "This verb converts Fahrenheit degrees to Celsius degrees.";
    return (tofloat(args[1]) - 32.0) / 1.8;
  endverb

  verb "C_to_F degC_to_degF" (this none this) owner: #36 flags: "rxd"
    ":C_to_F(INT|FLOAT <Celsius>) => FLOAT <Fahrenheit>";
    "This verb converts Celsius degrees to Fahrenheit degrees.";
    return tofloat(args[1]) * 1.8 + 32.0;
  endverb

  verb "convert" (this none this) owner: #36 flags: "rxd"
    ":convert(STR <units>, STR <units>) => FLOAT conversion factor | LIST errors.";
    "This verb attempts to compute the conversion factor between two sets of units. If the two inputs are of the same type (two speeds, two lengths, etc.), the value is returned. If the two inputs are not of the same type, a LIST is returned as follows: {1, {FLOAT <value>, STR <units>}. {FLOAT <value>, STR <units>}}. The 1 indicates that the two inputs were correctly formed. <value> is the conversion factor of the input into the basic <units>. This error output is useful for determining the basic structure and value of an unknown unit of measure. If either of the inputs can not be broken down to known units, a LIST is returned as follows: {0, STR <bad input>}.";
    "";
    "The format of the input strings is fairly straight forward: any multiplicative combination of units, ending in an optional digit to represent that unit is raised to a power, the whole of which is preceeded by an initial value. Examples: \"100 kg m/sec2\", \"35 joules\", \"2000 furlongs/fortnight\"";
    "";
    "Some example uses:";
    ";$convert_utils:convert(\"2000 furlongs/fortnight\", \"mph\")";
    "=> 0.744047619047619";
    ";$convert_utils:convert(\"kilowatt hours\", \"joules\")";
    "=> 3600000.0";
    "";
    ";$convert_utils:convert(\"furlongs\", \"mph\")";
    "=> {1, {201.168, \"m\"}, {044704, \"m / s\"}}";
    "";
    ";$convert_utils:convert(\"junk\", \"meters\")";
    "=> {0, \"junk\"}";
    {havestr, wantstr} = args;
    {havenum, havestr} = $string_utils:first_word(havestr);
    havestr = $string_utils:trimr(tostr(havenum, " ", strsub(havestr, " ", "")));
    wantstr = strsub(wantstr, " ", "");
    "Preceeding three lines added by GD (#110777) on 23-June-2007 to stop an annoying error when you try to convert to/from things like 'fluid ounces'.";
    have = this:_do_convert(havestr);
    want = this:_do_convert(wantstr);
    if (have && want && have[2] == want[2])
      return have[1] / want[1];
    elseif (have && want)
      return {1, {have[1], this:_format_units(@have[2])}, {want[1], this:_format_units(@want[2])}};
    else
      return {0, have ? wantstr | havestr};
    endif
  endverb

  verb "_do_convert" (this none this) owner: #36 flags: "rxd"
    "THIS VERB IS NOT INTENDED FOR USER USAGE.";
    ":_do_convert is the workhorse of $convert_utils:convert and is based loosely upon the 'units' Perl script the ships with BSD Unix.";
    "Essentially, it breaks the input up into values and units, attempts to break each unit down into elementary (basic) units, modifies the value as it goes, until it has no more input or can not convert a unit into a basic unit.";
    instr = args[1];
    units = this.basic_units_template;
    value = 1.0;
    top = 1;
    "Ensure that the division mark is a spearate word.";
    instr = $string_utils:substitute(instr, {{"/", " / "}});
    while (instr)
      "Grab the next word to process";
      {first, instr} = $string_utils:first_word(instr);
      if (first == "/")
        "Now we're working with values under the division mark - units with negative exponents.";
        top = 1 - top;
        continue;
      elseif (match(first, "|"))
        "The word was a value expressed as a ratio. Compute the ratio and adjust the value accordingly.";
        value = this:_do_value(first, value, top);
        continue;
      elseif ($string_utils:is_integer(first) || $string_utils:is_float(first))
        "The word was a value. Adjust the accumulated value accordingly.";
        value = top ? value * tofloat(first) | value / tofloat(first);
        continue;
      elseif (match(first, "[0-9]$"))
        "The word ends with a digit, but isn't a value. It must be a powered unit. Expand it: cm3 => cm cm cm";
        subs = match(first, "%([a-zA-Z]+%)%([0-9]+%)");
        first = substitute("%1", subs);
        power = toint(substitute("%2", subs));
        while (power > 0)
          instr = first + " " + instr;
          power = power - 1;
        endwhile
        continue;
      else
        "Check to see if the word starts with one or more metric prefix and attempt to evaluate the prefix.";
        {first, value, top} = this:_try_metric_prefix(first, value, top);
        "Check to see if we have a basic unit. If so, adjust the apropriate unit count.";
        if (index = first in this.basic_units)
          units[index][2] = top ? units[index][2] + 1 | units[index][2] - 1;
          continue;
        elseif (prop = `this.(first) ! E_PROPNF => 0')
          "Check to see if this is a known unit. If so, convert it and adjust the value and units.";
          result = this:_do_convert(prop);
          value = top ? value * result[1] | value / result[1];
          for i in [1..length(units)]
            units[i][2] = top ? units[i][2] + result[2][i][2] | units[i][2] - result[2][i][2];
          endfor
          continue;
        elseif (first[$] == "s")
          "Check to see if this is a normal 's'-ending plural, and try to do the above checks again.";
          temp = first[1..$ - 1];
          if (index = temp in this.basic_units)
            units[index][2] = top ? units[index][2] + 1 | units[index][2] - 1;
            continue;
          elseif (prop = `this.(temp) ! E_PROPNF => 0')
            result = this:_do_convert(prop);
            value = top ? value * result[1] | value / result[1];
            for i in [1..length(units)]
              units[i][2] = top ? units[i][2] + result[2][i][2] | units[i][2] - result[2][i][2];
            endfor
            continue;
          endif
        endif
        "We were unable to find any conversion for the current word, so halt all operation and return 0.";
        return 0;
      endif
    endwhile
    "We were able to successfully convert each part of the input. Return the equivalent value and units.";
    return {value, units};
  endverb

  verb "_try_metric_prefix" (this none this) owner: #36 flags: "rxd"
    "THIS VERB IS NOT INTENDED FOR USER USAGE.";
    ":_try_metric_prefix runs through the metrix multipliers and tries to match them against the beginning of the input string. If successful, the given value is adjusted appropritately, and the input string is modified. The verb loops until there are no more prefix matches. (Hence, \"kilodecameter\" can be matched with only one verb call.";
    "If anyone knows of other possibilities here, please let me know.";
    {first, value, top} = args;
    while (1)
      if (subs = match(first, "^yocto%(.*%)"))
        first = substitute("%1", subs);
        value = top ? value / 1e+24 | value * 1e+24;
        continue;
      endif
      if (subs = match(first, "^zepto%(.*%)"))
        first = substitute("%1", subs);
        value = top ? value / 1e+21 | value * 1e+21;
        continue;
      endif
      if (subs = match(first, "^atto%(.*%)"))
        first = substitute("%1", subs);
        value = top ? value / 1e+18 | value * 1e+18;
        continue;
      endif
      if (subs = match(first, "^femto%(.*%)"))
        first = substitute("%1", subs);
        value = top ? value / 1e+15 | value * 1e+15;
        continue;
      endif
      if (subs = match(first, "^pico%(.*%)"))
        first = substitute("%1", subs);
        value = top ? value / 1000000000000.0 | value * 1000000000000.0;
        continue;
      endif
      if (subs = match(first, "^nano%(.*%)"))
        first = substitute("%1", subs);
        value = top ? value / 1000000000.0 | value * 1000000000.0;
        continue;
      endif
      if (match(first, "^micron"))
        break;
      endif
      if (subs = match(first, "^micro%(.*%)"))
        first = substitute("%1", subs);
        value = top ? value / 1000000.0 | value * 1000000.0;
        continue;
      endif
      if (subs = match(first, "^milli%(.*%)"))
        first = substitute("%1", subs);
        value = top ? value / 1000.0 | value * 1000.0;
        continue;
      endif
      if (subs = match(first, "^centi%(.*%)"))
        first = substitute("%1", subs);
        value = top ? value / 100.0 | value * 100.0;
        continue;
      endif
      if (subs = match(first, "^deci%(.*%)"))
        first = substitute("%1", subs);
        value = top ? value / 10.0 | value * 10.0;
        continue;
      endif
      if (subs = match(first, "^%(deca%|deka%)%(.*%)"))
        first = substitute("%2", subs);
        value = !top ? value / 10.0 | value * 10.0;
        continue;
      endif
      if (subs = match(first, "^hecto%(.*%)"))
        first = substitute("%1", subs);
        value = !top ? value / 100.0 | value * 100.0;
        continue;
      endif
      if (subs = match(first, "^kilo%(.*%)"))
        first = substitute("%1", subs);
        value = !top ? value / 1000.0 | value * 1000.0;
        continue;
      endif
      if (subs = match(first, "^mega%(.*%)"))
        first = substitute("%1", subs);
        value = !top ? value / 1000000.0 | value * 1000000.0;
        continue;
      endif
      if (subs = match(first, "^giga%(.*%)"))
        first = substitute("%1", subs);
        value = !top ? value / 1000000000.0 | value * 1000000000.0;
        continue;
      endif
      if (subs = match(first, "^tera%(.*%)"))
        first = substitute("%1", subs);
        value = !top ? value / 1000000000000.0 | value * 1000000000000.0;
        continue;
      endif
      if (subs = match(first, "^peta%(.*%)"))
        first = substitute("%1", subs);
        value = !top ? value / 1e+15 | value * 1e+15;
        continue;
      endif
      if (subs = match(first, "^exa%(.*%)"))
        first = substitute("%1", subs);
        value = !top ? value / 1e+18 | value * 1e+18;
        continue;
      endif
      if (subs = match(first, "^zetta%(.*%)"))
        first = substitute("%1", subs);
        value = !top ? value / 1e+21 | value * 1e+21;
        continue;
      endif
      if (subs = match(first, "^yotta%(.*%)"))
        first = substitute("%1", subs);
        value = !top ? value / 1e+24 | value * 1e+24;
        continue;
      endif
      break;
    endwhile
    return {first, value, top};
  endverb

  verb "_format_units" (this none this) owner: #36 flags: "rxd"
    "THIS VERB IS NOT INTENDED FOR USER USAGE.";
    ":_format_units takes the associative list of units and powers and construct a more user friendly string.";
    top = bottom = "";
    for pair in (args)
      if (pair[2] > 0)
        top = tostr(top, " ", pair[1], pair[2] > 1 ? pair[2] | "");
      elseif (pair[2] < 0)
        bottom = tostr(bottom, " ", pair[1], pair[2] < -1 ? -pair[2] | "");
      endif
    endfor
    if (bottom)
      return (top + " /" + bottom)[2..$];
    else
      return top[2..$];
    endif
  endverb

  verb "K_to_C degK_to_degC" (this none this) owner: #36 flags: "rxd"
    ":K_to_C (INT|FLOAT <Kelvin>) => FLOAT <Celcius>";
    "This verb converts Kelvin degrees to Celcius degrees.";
    return tofloat(args[1]) - 273.0;
  endverb

  verb "C_to_K degC_to_degK" (this none this) owner: #36 flags: "rxd"
    ":C_to_K (INT|FLOAT <Celcius>) => FLOAT <Kelvin>";
    "This verb converts Celcius degrees to Kelvin degrees.";
    return tofloat(args[1]) + 273.0;
  endverb

  verb "F_to_R degF_to_degR" (this none this) owner: #36 flags: "rxd"
    ":F_to_R (INT|FLOAT <Fahrenheit>) => FLOAT <Rankine>";
    "This verb converts Fahrenheit degrees to Rankine degrees.";
    return tofloat(args[1]) + 459.67;
  endverb

  verb "R_to_F degR_to_degF" (this none this) owner: #36 flags: "rxd"
    ":R_to_F (INT|FLOAT <Rankine>) => FLOAT <Fahrenheit>";
    "This verb converts Rankine degrees to Fahrenheit degrees.";
    return tofloat(args[1]) - 459.67;
  endverb

  verb "_do_value" (this none this) owner: #36 flags: "rxd"
    "THIS VERB IS NOT INTENDED FOR USER USAGE.";
    ":_do_value takes a string of the form <number>|<number>, interprets it as a ratio, and applies that ratio to the incoming 'value' accordingly with the 'top' input, and returns it back to the calling verb.";
    {first, value, top} = args;
    {numer, denom} = $string_utils:explode(first, "|");
    return top ? value * tofloat(numer) / tofloat(denom) | value * tofloat(denom) / tofloat(numer);
  endverb

endobject
