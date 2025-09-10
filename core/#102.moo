object #102
  name: "ANSI Options"
  parent: #68
  location: #-1
  owner: #98
  readable: true
  override "key" = 0;

  override "aliases" = {"ANSI Options", "ao"};

  override "description" = "An option package for controlling and customizing the ANSI PC.";

  override "names" = {"colors", "blinking", "bold", "misc", "escape", "ignore", "no_connect_status", "extra", "backgrounds", "truecolor", "256"};

  override "_namelist" = "!colors!blinking!bold!misc!escape!ignore!no_connect_status!extra!backgrounds!truecolor!all!none!256!";

  override "extras" = {"all", "none"};

  override "namewidth" = 25;

  property "show_colors" (owner: #98, flags: "r") = {"Strip out standard 16 color sequences.", "Translate standard 16 color sequences."};

  property "show_backgrounds" (owner: #98, flags: "rc") = {"Strip out background color sequences.", "Translate background color sequences."};

  property "show_blinking" (owner: #98, flags: "r") = {"Do not show blinking sequences.", "Show blinking sequences."};

  property "show_bold" (owner: #98, flags: "r") = {"Do not show bright colors.", "Send bright colors."};

  property "show_misc" (owner: #98, flags: "r") = {"Do not send all the other ANSI sequences.", "Send all the other ANSI sequences."};

  property "show_all" (owner: #98, flags: "r") = {"Do not send any ANSI sequences.", "Send all ANSI sequences."};

  property "show_none" (owner: #98, flags: "r") = {"Send all ANSI sequences.", "Do not send any ANSI sequences."};

  property "type_escape" (owner: #98, flags: "r") = {2};

  property "show_ignore" (owner: #98, flags: "r") = {"See 'help ansi-options' for more information.", "Overriding other options and ignoring all ANSI codes."};

  property "show_no_connect_status" (owner: #98, flags: "r") = {"Don't show any status messages upon connecting.", "Show status messages about ANSI upon connecting."};

  property "show_extra" (owner: #98, flags: "r") = {"Do not send extra non-ANSI codes (such as beep).", "Send extra non-ANSI codes (such as beep)."};

  property "show_truecolor" (owner: #98, flags: "r") = {"Strip out 24-bit True Color sequences.", "Translate 24-bit True Color sequences."};

  property "show_approximate_256" (owner: #98, flags: "rc") = {"24-bit color sequences are not approximated to 8-bit sequences for older clients.", "24-bit color sequences are approximated to 8-bit sequences for older clients."};

  property "show_256" (owner: #98, flags: "rc") = {"Strip out Xterm 256 color sequences.", "Translate Xterm 256 color sequences."};

  verb "actual" (this none this) owner: #98 flags: "rxd"
    if (args[1] == "all")
      return {{"colors", a = args[2]}, {"backgrounds", a}, {"extra", a}, {"misc", a}, {"blinking", a}, {"bold", a}, {"ignore", 0}, {"truecolor", a}, {"256", a}};
    elseif (args[1] == "none")
      return {{"colors", a = !args[2]}, {"backgrounds", a}, {"extra", a}, {"misc", a}, {"blinking", a}, {"bold", a}, {"truecolor", a}, {"256", a}};
    else
      return {args};
    endif
  endverb

  verb "show_escape" (this none this) owner: #98 flags: "rxd"
    if (value = this:get(@args))
      return {value, {tostr("Send \"", value, "\" for the escape character.")}};
    else
      return {0, {"Use a character 27 as the escape character."}};
    endif
  endverb

  verb "parse_escape" (this none this) owner: #98 flags: "rxd"
    oname = args[1];
    raw = args[2];
    if (typeof(raw) == STR)
      return {oname, raw};
    else
      return "String value expected.";
    endif
  endverb

endobject
