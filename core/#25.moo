object #25
  name: "ANSI Color System"
  parent: #78
  location: #-1
  owner: #12
  readable: true
  override "key" = 0;

  override "aliases" = {"ANSI", "Color", "System", "ANSI Color System"};

  override "help_msg" = {"This is the ANSI system custom-built for Torchship by Mirage.", "", "Theory of Operation:", "  Tags in the form of \"`<function><###>]\" are inserted into strings as color functions are called. When these messages are sent to players, their :notify verb colorizes or strips out these soft tags, converting them finally into actual ANSI sequences.", "", "Why Tags?:", "  Many built-in functions don't support the ANSI escape sequence, making it inadvisable to work with directly until that text is being sent to the user. (Examples: generate_json, parse_json, and pcre_* functions.)", "", "Function Dictionary:", "  f   => foreground 8-bit color", "  b   => background 8-bit color", "  x   => xterm 256 foreground color", "  l   => blink", "  u   => underline", "  o   => bold", "  n   => terminate", "", "  ### will be either the 256 xterm color", "  0-7 with f or g means \"normal\" black-white", "  8-15 with f or g means \"bright\" black-white", "  l, y, o, n will always have a ### of 0."};

  property "red" (owner: #12, flags: "r") = "[31m";

  property "green" (owner: #12, flags: "r") = "[32m";

  property "yellow" (owner: #12, flags: "r") = "[33m";

  property "blue" (owner: #12, flags: "r") = "[34m";

  property "purple" (owner: #12, flags: "r") = "[35m";

  property "cyan" (owner: #12, flags: "r") = "[36m";

  property "normal" (owner: #12, flags: "r") = "[0m";

  property "inverse" (owner: #12, flags: "r") = "e[7m";

  property "underline" (owner: #12, flags: "r") = "[4m";

  property "bold" (owner: #12, flags: "r") = "[1m";

  property "brblack" (owner: #12, flags: "r") = "e[40m";

  property "brwhite" (owner: #12, flags: "r") = "[1m[37m";

  property "gold" (owner: #12, flags: "r") = "[33m";

  property "magenta" (owner: #12, flags: "r") = "[35m";

  property "bright" (owner: #12, flags: "r") = "[1m";

  property "unbright" (owner: #12, flags: "r") = "[22m";

  property "grey" (owner: #12, flags: "r") = "[1m[30m";

  property "white" (owner: #12, flags: "r") = "[37m";

  property "italic" (owner: #12, flags: "r") = "[3m";

  property "bgblue" (owner: #12, flags: "r") = "[44m";

  property "bgred" (owner: #12, flags: "r") = "[41m";

  property "bggreen" (owner: #12, flags: "r") = "[42m";

  property "bgcyan" (owner: #12, flags: "r") = "[46m";

  property "bgyellow" (owner: #12, flags: "r") = "[43m";

  property "escape" (owner: #12, flags: "r") = "";

  property "xterm256_colors" (owner: #12, flags: "r") = ["Black" -> 0, "BlackEbony" -> 234, "BlackFlat" -> 246, "BlackGrey" -> 16, "BlackJet" -> 238, "BlackLicorice" -> 237, "BlackMatte" -> 235, "BlackMidnight" -> 233, "BlackOnyx" -> 239, "BlackRaven" -> 236, "Blue" -> 4, "BlueAquamarine" -> 79, "BlueAquamarineLight" -> 86, "BlueBright" -> 12, "BlueBrilliant" -> 39, "BlueBrilliantMedium" -> 38, "BlueCadet" -> 72, "BlueCadetLight" -> 73, "BlueCornflower" -> 69, "BlueCyanLight" -> 51, "BlueCyanMedium" -> 50, "BlueDark" -> 18, "BlueDarkLight" -> 21, "BlueDeepSkyDark" -> 24, "BlueDeepSkyMedium" -> 25, "BlueDodger" -> 33, "BlueDodgerDark" -> 26, "BlueDodgerMedium" -> 27, "BlueLight" -> 45, "BlueMediumDark" -> 20, "BlueMostlyDark" -> 19, "BlueNavy" -> 17, "BluePaleTurqouise" -> 159, "BluePeriwinkle" -> 153, "BlueRoyal" -> 63, "BlueSeaGreen" -> 37, "BlueSeaGreenMedium" -> 115, "BlueSilver" -> 23, "BlueSky" -> 74, "BlueSkyDark" -> 31, "BlueSkyLightSilver" -> 109, "BlueSkyLightSilverBright" -> 110, "BlueSkyMedium" -> 32, "BlueSkyPale" -> 117, "BlueSkyPurple" -> 111, "BlueSlate" -> 99, "BlueSlateDark" -> 62, "BlueSlateDeep" -> 61, "BlueSlateGrey" -> 123, "BlueSlateLight" -> 158, "BlueSteel" -> 75, "BlueSteelDark" -> 68, "BlueSteelDeep" -> 67, "BlueSteelLight" -> 81, "BlueTurquoise" -> 30, "BlueTurquoiseMedium" -> 80, "Brown" -> 94, "BrownBeige" -> 179, "BrownChocolate" -> 52, "BrownCyan" -> 152, "BrownGrey" -> 139, "BrownPink" -> 95, "BrownRosy" -> 138, "BrownSalmon" -> 137, "BrownSeaGreen" -> 108, "BrownSteelBlue" -> 146, "BrownTan" -> 180, "Cyan" -> 6, "CyanBright" -> 14, "CyanDark" -> 36, "CyanLightGrey" -> 195, "CyanTurqouise" -> 44, "Green" -> 2, "GreenAquamarine" -> 122, "GreenAutumDark" -> 29, "GreenBright" -> 10, "GreenChartreuse" -> 112, "GreenChartreuseBright" -> 118, "GreenChartreuseDark" -> 76, "GreenChartreuseDeep" -> 70, "GreenChartreuseMedim" -> 82, "GreenForestBright" -> 46, "GreenForestCyan" -> 43, "GreenForestDark" -> 28, "GreenForestLight" -> 41, "GreenForestMedium" -> 40, "GreenForestMediumDark" -> 34, "GreenForestMostlyDark" -> 22, "GreenForstBlue" -> 42, "GreenHoneyDew" -> 194, "GreenJade" -> 35, "GreenLight" -> 119, "GreenLightMedium" -> 120, "GreenLightSea" -> 193, "GreenNeonMatte" -> 156, "GreenOlive" -> 64, "GreenOliveDark" -> 192, "GreenOliveLight" -> 113, "GreenOliveMatte" -> 107, "GreenOliveMedium" -> 149, "GreenPale" -> 77, "GreenPaleLight" -> 121, "GreenPaleMedium" -> 114, "GreenSea" -> 83, "GreenSeaDark" -> 78, "GreenSeaDeep" -> 65, "GreenSeaDeepDark" -> 71, "GreenSeaLight" -> 85, "GreenSeaMedium" -> 84, "GreenSlate" -> 157, "GreenSpringBright" -> 47, "GreenSpringMatte" -> 48, "GreenSpringMedium" -> 49, "GreenTurquoisePale" -> 66, "GreenYellow" -> 154, "GreenYellowLight" -> 155, "Grey" -> 8, "GreyAlmostBlack" -> 232, "GreyCharcoal" -> 251, "GreyDarkSlate" -> 247, "GreyDeep" -> 59, "GreyGreenSea" -> 151, "GreyHeather" -> 244, "GreyLight" -> 253, "GreyLightAlt" -> 252, "GreyLightSlate" -> 103, "GreyPink" -> 188, "GreyPurpleDeep" -> 60, "GreySilver" -> 240, "GreySilverAlt" -> 241, "GreySilverLight" -> 242, "GreySilverLighter" -> 243, "GreySilverMatte" -> 245, "GreySlate" -> 250, "GreySlateAlt" -> 249, "GreySlateBlue" -> 189, "GreySlateDark" -> 87, "GreySlateMedium" -> 116, "GreySlightlyDarkSlate" -> 248, "GreyVeryLight" -> 254, "GreyWheat" -> 102, "GreyWhite" -> 231, "Magenta" -> 5, "MagentaBright" -> 13, "MagentaBrightLight" -> 200, "MagentaBrightMedium" -> 201, "MagentaDark" -> 53, "MagentaDarkPink" -> 90, "MagentaDarkPurple" -> 91, "MagentaDarkViolet" -> 92, "MagentaDeepPink" -> 199, "MagentaHotPink" -> 132, "MagentaMediumAlt" -> 135, "MagentaOrchid" -> 207, "MagentaOrchidBright" -> 171, "MagentaOrchidDark" -> 133, "MagentaOrchidDeep" -> 170, "MagentaOrchidMedium" -> 134, "MagentaPinkDeep" -> 89, "MagentaPlum" -> 176, "MagentaPlumBright" -> 219, "MagentaPlumMedium" -> 183, "MagentaPurple" -> 54, "MagentaPurpleBlue" -> 104, "MagentaPurpleBright" -> 141, "MagentaPurpleDark" -> 55, "MagentaPurpleLavendar" -> 98, "MagentaPurpleLight" -> 57, "MagentaPurpleMedium" -> 56, "MagentaPurpleMediumMatte" -> 140, "MagentaPurpleOrchidDark" -> 213, "MagentaPurpleOrchidLight" -> 212, "MagentaPurplePlum" -> 96, "MagentaPurplePlumMedium" -> 97, "MagentaPurplePosh" -> 129, "MagentaPurpleSlate" -> 93, "MagentaPurpleSlateLight" -> 105, "MagentaRuby" -> 163, "MagentaRubyBright" -> 164, "MagentaRubyLight" -> 165, "MagentaSteelBlue" -> 147, "MagentaThistle" -> 182, "MagentaThistleBright" -> 225, "MagentaViolet" -> 177, "MagentaVioletDark" -> 128, "MagentaVioletLight" -> 127, "MagentaVioletMedium" -> 126, "OrangeBlood" -> 130, "OrangeBloodLight" -> 166, "OrangeBright" -> 214, "OrangeBrown" -> 58, "OrangeDark" -> 208, "OrangeLightSalmon" -> 216, "OrangeMedium" -> 172, "OrangeTangerine" -> 215, "Red" -> 1, "RedBlood" -> 160, "RedBloodBright" -> 196, "RedBloodMatte" -> 197, "RedBright" -> 9, "RedCherry" -> 161, "RedCoralLight" -> 210, "RedDark" -> 88, "RedDeepPink" -> 198, "RedHotPink" -> 205, "RedHotPinkBright" -> 206, "RedIndian" -> 203, "RedIndianMagenta" -> 131, "RedIndianMedium" -> 167, "RedIndianPink" -> 204, "RedMistyRose" -> 181, "RedMistyRoseLight" -> 224, "RedOrange" -> 202, "RedPinkDark" -> 175, "RedPinkDeep" -> 125, "RedPinkHot" -> 169, "RedPinkHotDark" -> 168, "RedPinkLight" -> 217, "RedPinkMatte" -> 174, "RedPinkMedium" -> 218, "RedPinkWhite" -> 223, "RedRuby" -> 162, "RedSalmon" -> 173, "RedSalmonMatte" -> 209, "RedSector" -> 124, "RedVioletPale" -> 211, "White" -> 7, "WhiteBright" -> 15, "WhiteSilver" -> 255, "Yellow" -> 3, "YellowBright" -> 11, "YellowBrown" -> 100, "YellowCanary" -> 226, "YellowCanaryMedium" -> 227, "YellowGold" -> 220, "YellowGoldBright" -> 221, "YellowGolden" -> 178, "YellowGoldenBright" -> 190, "YellowGoldenDark" -> 142, "YellowGoldenLight" -> 186, "YellowGoldenMedium" -> 191, "YellowGoldenRodDark" -> 136, "YellowGoldenVeryLight" -> 187, "YellowGoldMatte" -> 222, "YellowGreenLight" -> 148, "YellowGreenSea" -> 150, "YellowKhaki" -> 185, "YellowKhakiDark" -> 143, "YellowKhakiLight" -> 228, "YellowLight" -> 184, "YellowNavajoWhite" -> 144, "YellowNavajoWhiteLight" -> 145, "YellowWheat" -> 229, "YellowWheatDark" -> 101, "YellowWheatLight" -> 230, "YellowWheatMedium" -> 106];

  property "8bit_colors" (owner: #12, flags: "r") = ["black" -> 0, "blue" -> 4, "brblack" -> 8, "brblue" -> 12, "brcyan" -> 14, "brgreen" -> 10, "brmagenta" -> 13, "brred" -> 9, "brwhite" -> 15, "bryellow" -> 11, "cyan" -> 6, "green" -> 2, "magenta" -> 5, "red" -> 1, "white" -> 7, "yellow" -> 3];

  property "8bit_colors_reverse_map" (owner: #12, flags: "r") = [0 -> "black", 1 -> "red", 2 -> "green", 3 -> "yellow", 4 -> "blue", 5 -> "magenta", 6 -> "cyan", 7 -> "white", 8 -> "brblack", 9 -> "brred", 10 -> "brgreen", 11 -> "bryellow", 12 -> "brblue", 13 -> "brmagenta", 14 -> "brcyan", 15 -> "brwhite"];

  property "blink" (owner: #12, flags: "r") = "[5m";

  property "brred" (owner: #12, flags: "r") = "[1m[31m";

  property "brgreen" (owner: #12, flags: "r") = "[1m[32m";

  property "bryellow" (owner: #12, flags: "r") = "[1m[33m";

  property "brblue" (owner: #12, flags: "r") = "[1m[34m";

  property "brmagenta" (owner: #12, flags: "r") = "[1m[35m";

  property "brcyan" (owner: #12, flags: "r") = "[1m[36m";

  property "all_Regexp" (owner: #12, flags: "r") = "`[fbxluon][0-9][0-9][0-9]]";

  property "rainbow_colors" (owner: #12, flags: "r") = {"red", "brred", "yellow", "bryellow", "green", "brgreen", "cyan", "brcyan", "magenta", "brmagenta"};

  property "haircolor_table" (owner: #12, flags: "r") = {{"normal", "jet black"}, {"normal", "raven black"}, {"normal", "midnight black"}, {"normal", "black"}, {"normal", "gray"}, {"cyan", "teal"}, {"normal", "chocolate"}, {"normal", "light brown"}, {"normal", "dark brown"}, {"normal", "auburn"}, {"yellow", "dirty blond"}, {"bryellow", "blond"}, {"bryellow", "honey blond"}, {"green", "olive green"}, {"green", "jade green"}, {"magenta", "lavender"}, {"magenta", "purple"}, {"blue", "navy blue"}, {"brblue", "sapphire blue"}, {"brblue", "azure blue"}, {"brblue", "brilliant blue"}, {"brcyan", "periwinkle blue"}, {"brcyan", "light blue"}, {"green", "evergreen"}, {"green", "green"}, {"yellow", "mustard"}, {"red", "burgundy red"}, {"red", "maroon"}, {"red", "crimson"}, {"red", "dark red"}, {"brgreen", "pear green"}, {"brgreen", "lime green"}, {"brgreen", "emerald green"}, {"brgreen", "chartreuse"}, {"brgreen", "neon green"}, {"brmagenta", "pink"}, {"brmagenta", "bright purple"}, {"brmagenta", "brilliant fuchsia"}, {"brgold", "golden blond"}, {"gold", "gold"}, {"brgold", "bright gold"}, {"bryellow", "canary yellow"}, {"bryellow", "bright yellow"}, {"brwhite", "platinum blond"}, {"brred", "bright orange"}, {"brred", "tangerine"}, {"brred", "ruby red"}, {"brred", "cherry red"}, {"brred", "bright red"}, {"brwhite", "bright white"}};

  property "clothing_table" (owner: #12, flags: "r") = {{"normal", "brown"}, {"normal", "flat black"}, {"normal", "matte black"}, {"normal", "licorice black"}, {"normal", "jet black"}, {"normal", "ebony"}, {"normal", "raven black"}, {"normal", "midnight black"}, {"normal", "onyx black"}, {"normal", "black"}, {"normal", "charcoal gray"}, {"normal", "gray"}, {"normal", "heather gray"}, {"normal", "beige"}, {"cyan", "teal"}, {"normal", "chocolate"}, {"green", "olive green"}, {"green", "jade green"}, {"magenta", "lavender"}, {"magenta", "purple"}, {"blue", "navy blue"}, {"brblue", "sapphire blue"}, {"brblue", "azure blue"}, {"brblue", "brilliant blue"}, {"brcyan", "periwinkle blue"}, {"brcyan", "light blue"}, {"green", "evergreen"}, {"green", "green"}, {"yellow", "mustard"}, {"red", "burgundy red"}, {"red", "maroon"}, {"red", "crimson"}, {"red", "dark red"}, {"brgreen", "pear green"}, {"brgreen", "lime green"}, {"brgreen", "emerald green"}, {"brgreen", "chartreuse"}, {"brgreen", "neon green"}, {"brmagenta", "bright purple"}, {"brmagenta", "brilliant fuchsia"}, {"bryellow", "golden"}, {"yellow", "gold"}, {"bryellow", "bright gold"}, {"bryellow", "canary yellow"}, {"bryellow", "bright yellow"}, {"brred", "bright orange"}, {"brred", "tangerine"}, {"brred", "ruby red"}, {"brred", "cherry red"}, {"brred", "bright red"}, {"brwhite", "bright white"}, {"white", "silver"}};

  property "pearl_colors" (owner: #12, flags: "r") = {212, 176, 140, 104, 68, 32, 68, 104, 140, 176};

  property "chameleon_colors" (owner: #12, flags: "r") = {40, 41, 42, 43, 44, 45, 44, 43, 42, 41};

  property "golden_colors" (owner: #12, flags: "r") = {220, 221, 222, 223, 224, 225, 224, 223, 222, 221};

  property "mood_colors" (owner: #12, flags: "r") = {129, 128, 127, 126, 125, 124, 125, 126, 127, 128};

  property "transhumanist_colors" (owner: #12, flags: "r") = {219, 183, 147, 111, 75, 39, 75, 111, 147, 183};

  property "silvery_colors" (owner: #12, flags: "r") = {244, 245, 246, 247, 248, 249, 248, 247, 246, 245};

  property "coral_colors" (owner: #12, flags: "r") = {177, 176, 175, 174, 173, 172, 173, 174, 175, 176};

  property "goth_colors" (owner: #12, flags: "r") = {237, 238, 239, 240, 241, 242, 241, 240, 239, 238};

  property "crimson_colors" (owner: #12, flags: "r") = {160, 124, 1, 88, 52, 88, 1, 124};

  property "sapphire_colors" (owner: #12, flags: "r") = {20, 19, 18, 4, 17, 4, 18, 19};

  property "black" (owner: #12, flags: "r") = "[0;30m";

  verb "red brred blue brblue white brwhite green brgreen brblack yellow bryellow normal cyan brcyan magenta brmagenta underline bgnormal bgblue bgred bggreen bgcyan bgmagenta bgwhite bgyellow gold brgold bright none bgred bgblue bgyellow bgcyan bggreen black" (this none this) owner: #36 flags: "rxd"
    "colorize a string, or set an escape code/color without closing the color";
    ":red blue ..etc(?STR text) => STR";
    codes = "";
    if (verb == "bright")
      verb = "brwhite";
    endif
    return this:terminate_normal(this:get_tag(verb), @args);
  endverb

  verb "strip_tags" (this none this) owner: #36 flags: "rxd"
    ":strip_tags(STR line) => STR line without ANSI tags";
    {line} = args;
    if (!line)
      return line;
    endif
    return pcre_replace(tostr(line), tostr("s/", this.all_regexp, "//gi"));
  endverb

  verb "get_tag" (this none this) owner: #36 flags: "rxd"
    ":get_tag(STR color[, BOOL xterm]) => STR tag";
    "  generates an ANSI color tag.";
    {color, ?xterm = $false} = args;
    if (color == "normal")
      return tostr("`n000]");
    elseif (color == "blink")
      return tostr("`l000]");
    elseif (color == "underline")
      return tostr("`u000]");
    elseif (color == "bold")
      return tostr("`o000]");
    endif
    if (xterm)
      mode = "x";
      color = $su:right(this.xterm256_colors[color], 3, "0");
    else
      mode = "f";
      if (color[1..2] == "bg")
        mode = "b";
        color = color[3..$];
      endif
      try
        color = $su:right(this.("8bit_colors")[color], 3, "0");
      except e (ANY)
        raise(E_ARGS, tostr("Unable to find a color by the name of '", color, "'."));
      endtry
    endif
    return tostr("`", mode, color, "]");
  endverb

  verb "convert_tags_to_color" (this none this) owner: #36 flags: "rxd"
    ":convert_tags_to_color(STR line) => STR of colorized text";
    {line, @more} = args;
    if (!line)
      return line;
    endif
    offset = 0;
    ansi_stack = {};
    for ansi_match in (pcre_match(tostr(line), $ansi.all_regexp, $true))
      
      tag = ansi_match["0"]["match"];
      ansi_sequence = $ansi.escape;
      if (tag[2] == "x")
        ansi_sequence = tostr(ansi_sequence, "[38;5;", toint(tag[3..5]), "m");
      elseif (tag[2] == "n")
        ansi_sequence = tostr(ansi_sequence, $ansi.normal);
      elseif (tag[2] in {"f", "b"})
        ansi_sequence = tostr(ansi_sequence, $ansi.($ansi.("8bit_colors_reverse_map")[toint(tag[3..5])]));
      elseif (tag[2] == "l")
        ansi_sequence = tostr(ansi_sequence, $ansi.blink);
      elseif (tag[2] == "o")
        ansi_sequence = tostr(ansi_sequence, $ansi.bold);
      elseif (tag[2] == "u")
        ansi_sequence = tostr(ansi_sequence, $ansi.underline);
      endif
      position = ansi_match["0"]["position"];
      if (tag == "`n000]")
        {last_sequence, ansi_stack} = `$lu:pop(ansi_stack) ! ANY => {"", {}}';
        {last_sequence, ansi_stack} = `$lu:pop(ansi_stack) ! ANY => {"", {}}';
        ansi_sequence = tostr(ansi_sequence, last_sequence);
        if (last_sequence)
          ansi_stack = {@ansi_stack, ansi_sequence};
        endif
      else
        if (ansi_stack)
          ansi_sequence = tostr($ansi.escape, $ansi.normal, ansi_sequence);
        endif
        ansi_stack = {@ansi_stack, ansi_sequence};
      endif
      line[position[1] - offset..position[2] - offset] = ansi_sequence;
      offset = offset + (length(tag) - length(ansi_sequence));
    endfor
    if (ansi_stack)
      line = tostr(line, $ansi.escape, $ansi.normal);
    endif
    return line;
  endverb

  verb "BlackEbony BlackFlat BlackGrey BlackJet BlackLicorice BlackMatte BlackMidnight BlackOnyx BlackRaven BlueAquamarine BlueAquamarineLight BlueBright BlueBrilliant BlueBrilliantMedium BlueCadet BlueCadetLight BlueCornflower BlueCyanLight BlueCyanMedium BlueDark BlueDarkLight BlueDeepSkyDark BlueDeepSkyMedium BlueDodger BlueDodgerDark BlueDodgerMedium BlueLight BlueMediumDark BlueMostlyDark BlueNavy BluePaleTurqouise BluePeriwinkle BlueRoyal BlueSeaGreen BlueSeaGreenMedium BlueSilver BlueSky BlueSkyDark BlueSkyLightSilver BlueSkyLightSilverBright BlueSkyMedium BlueSkyPale BlueSkyPurple BlueSlate BlueSlateDark BlueSlateDeep BlueSlateGrey BlueSlateLight BlueSteel BlueSteelDark BlueSteelDeep BlueSteelLight BlueTurquoise BlueTurquoiseMedium Brown BrownBeige BrownChocolate BrownCyan BrownGrey BrownPink BrownRosy BrownSalmon BrownSeaGreen BrownSteelBlue BrownTan CyanBright CyanDark CyanLightGrey CyanTurqouise GreenAquamarine GreenAutumDark GreenBright GreenChartreuse GreenChartreuseBright GreenChartreuseDark GreenChartreuseDeep GreenChartreuseMedim GreenForestBright GreenForestCyan GreenForestDark GreenForestLight GreenForestMedium GreenForestMediumDark GreenForestMostlyDark GreenForstBlue GreenHoneyDew GreenJade GreenLight GreenLightMedium GreenLightSea GreenNeonMatte GreenOlive GreenOliveDark GreenOliveLight GreenOliveMatte GreenOliveMedium GreenPale GreenPaleLight GreenPaleMedium GreenSea GreenSeaDark GreenSeaDeep GreenSeaDeepDark GreenSeaLight GreenSeaMedium GreenSlate GreenSpringBright GreenSpringMatte GreenSpringMedium GreenTurquoisePale GreenYellow GreenYellowLight GreyAlmostBlack GreyCharcoal GreyDarkSlate GreyDeep GreyGreenSea GreyHeather GreyLight GreyLightAlt GreyLightSlate GreyPink GreyPurpleDeep GreySilver GreySilverAlt GreySilverLight GreySilverLighter GreySilverMatte GreySlate GreySlateAlt GreySlateBlue GreySlateDark GreySlateMedium GreySlightlyDarkSlate GreyVeryLight GreyWheat GreyWhite MagentaBright MagentaBrightLight MagentaBrightMedium MagentaDark MagentaDarkPink MagentaDarkPurple MagentaDarkViolet MagentaDeepPink MagentaHotPink MagentaMediumAlt MagentaOrchid MagentaOrchidBright MagentaOrchidDark MagentaOrchidDeep MagentaOrchidMedium MagentaPinkDeep MagentaPlum MagentaPlumBright MagentaPlumMedium MagentaPurple MagentaPurpleBlue MagentaPurpleBright MagentaPurpleDark MagentaPurpleLavendar MagentaPurpleLight MagentaPurpleMedium MagentaPurpleMediumMatte MagentaPurpleOrchidDark MagentaPurpleOrchidLight MagentaPurplePlum MagentaPurplePlumMedium MagentaPurplePosh MagentaPurpleSlate MagentaPurpleSlateLight MagentaRuby MagentaRubyBright MagentaRubyLight MagentaSteelBlue MagentaThistle MagentaThistleBright MagentaViolet MagentaVioletDark MagentaVioletLight MagentaVioletMedium OrangeBlood OrangeBloodLight OrangeBright OrangeBrown OrangeDark OrangeLightSalmon OrangeMedium OrangeTangerine RedBlood RedBloodBright RedBloodMatte RedBright RedCherry RedCoralLight RedDark RedDeepPink RedHotPink RedHotPinkBright RedIndian RedIndianMagenta RedIndianMedium RedIndianPink RedMistyRose RedMistyRoseLight RedOrange RedPinkDark RedPinkDeep RedPinkHot RedPinkHotDark RedPinkLight RedPinkMatte RedPinkMedium RedPinkWhite RedRuby RedSalmon RedSalmonMatte RedSector RedVioletPale WhiteBright WhiteSilver YellowBright YellowBrown YellowCanary YellowCanaryMedium YellowGold YellowGoldBright YellowGolden YellowGoldenBright YellowGoldenDark YellowGoldenLight YellowGoldenMedium YellowGoldenRodDark YellowGoldenVeryLight YellowGoldMatte YellowGreenLight YellowGreenSea YellowKhaki YellowKhakiDark YellowKhakiLight YellowLight YellowNavajoWhite YellowNavajoWhiteLight YellowWheat YellowWheatDark YellowWheatLight YellowWheatMedium Grey" (this none this) owner: #36 flags: "rxd"
    return this:terminate_normal(this:get_tag(verb, $true), @args);
  endverb

  verb "length" (this none this) owner: #36 flags: "rxd"
    return length(this:strip_tags(@args));
  endverb

  verb "terminate_normal" (this none this) owner: #36 flags: "rxd"
    return tostr(@args, "`n000", "]");
  endverb

  verb "cutoff" (this none this) owner: #36 flags: "rxd"
    ":cutoff (STR string, NUM start, NUM end) => STR";
    "Acts like: string[start..end] but ignores $xterm256 tags.";
    {string, start, end, ?terminate = 0} = args;
    COLOR_TAG_LENGTH = 6;
    try
      info = this:_cutoff_locs(string, start, end, terminate, 0);
      {sstart, send} = info;
      out = string[sstart..send];
      if ((endchange = send - end) > 0)
        "tags adjusted the end of the string";
        tags = endchange / COLOR_TAG_LENGTH;
        for i in [1..tags]
          out = this:terminate_normal(out);
        endfor
      endif
      return out;
    except (E_TYPE)
      return string;
    endtry
  endverb

  verb "_cutoff_locs" (this none this) owner: #36 flags: "rxd"
    ":cutoff_locs (STR string,NUM start,NUM end[,NUM extra][, NUM suspendok]) => {nstart, nend}";
    "Takes <start> and <end>, fixes them to compensate for the ANSI codes, and";
    "returns them.  If <extra> is provided and true, <nend> will include the";
    "codes after the ending letter.";
    {string, start, end, ?extra = 0, ?suspendok = $false} = args;
    if (typeof(string) != STR)
      return E_INVARG;
    elseif (start > end)
      return {start, end};
    endif
    i = begin = 0;
    x = 1;
    reg = this.all_regexp;
    l = length(string);
    while (x <= l)
      suspendok && (ticks_left() < 1000 || seconds_left() < 2) && suspend(0);
      if (m = match(string, reg))
        i = i + (m[1] - 1);
        if (!begin && i + 1 >= start)
          begin = x + m[1] - i + start - 2;
          if (end == "$")
            return {begin, l};
          endif
        endif
        if (begin && i - extra >= end)
          return {begin, x + m[1] - i + end - 2};
        endif
        x = x + m[2];
        string[1..m[2]] = "";
      else
        return {begin || x - i + start - 1, end == "$" ? l | x - i + end - 1};
      endif
    endwhile
    return end == i && begin ? {begin, l} | E_RANGE;
  endverb

  verb "rainbow brrainbow pearl golden mood transhumanist" (this none this) owner: #36 flags: "rxd"
    {strg, ?colors = this.(tostr(verb[1..2] == "br" ? verb[3..$] | verb, "_colors"))} = args;
    for i in [1..length(colors)]
      colors[i] = typeof(colors[i]) == INT ? $xterm:get_name(colors[i]) | colors[i];
    endfor
    chars = $su:char_list(strg);
    i = 1;
    for x in [1..length(chars)]
      
      chars[x] = $ansi:(colors[i])(chars[x]);
      i = i >= length(colors) ? 1 | i + 1;
    endfor
    return $su:from_list(chars, "");
  endverb

  verb "christmas brchristmas" (this none this) owner: #36 flags: "rxd"
    {strg} = args;
    return this:stripped(strg, "brred", "brgreen");
  endverb

  verb "blink" (this none this) owner: #36 flags: "rxd"
    return this:terminate_normal(this:get_tag(verb), @args);
  endverb

  verb "bold" (this none this) owner: #12 flags: "rxd"
    return this:terminate_normal(this:get_tag(verb), @args);
  endverb

endobject
