object #95
  name: "Slither's Code Scanner"
  parent: #78
  location: #-1
  owner: #2
  readable: true
  override "key" = 0;

  override "aliases" = {"Slither's Code Scanner", "Code Scanner", "Scanner", "Code"};

  override "help_msg" = {"MOO Code Scanner 1.1 by Brendan Butts <slither@sindome.org>", "", "Github: https://github.com/SevenEcks/lambda-moo-programming", "", "Usage: $code_scanner:scan_for_issues(OBJ, verbname)", "Usage: $code_scanner:display_issues($code_scanner:scan_for_issues(OBJ, verbname))", "", "If you integrate this with your @Program verb I recommend you make a copy of it first and test on that just in case! But you can always use .program if you mess up!"};

  property "integration_msg" (owner: #12, flags: "r") = "%T is hard at work, scanning dat code.";

  verb "scan_for_issues" (this none this) owner: #36 flags: "rxd"
    "MOO Code Scanner 1.1 by Brendan Butts <slither@sindome.org>";
    "GitHub for this: https://github.com/SevenEcks/lambda-moo-programming";
    ":scan_for_issues(OBJ object, STR verbname, ?LIST options, ?LIST code";
    "scan the provided object:verbname's code for possible issues";
    {object, verbname, ?options = {}, ?code = {}} = args;
    if (!code)
      code = verb_code(object, verbname);
    endif
    verb_args = verb_args(object, verbname);
    "max length before we start warning it's too long";
    MAX_LENGTH_WARNING = 40;
    "max nesting before we start warning";
    MAX_NESTING_WARNING = 2;
    "what do the args of an internal variable look like?";
    internal_args = {"this", "none", "this"};
    warnings = {};
    max_nest = 0;
    open_ifs = 0;
    open_fors = 0;
    open_whiles = 0;
    forks = 0;
    "first real line of code, not a comment";
    first_real_line = 0;
    "count of what line we are on";
    count = 0;
    "is this an internal (tnt) verb?";
    internal = internal_args == verb_args ? 1 | 0;
    for line in (code)
      
      count = count + 1;
      "check for opening comments";
      if (count == 1 && internal && !this:match_comment(line))
        warnings = {@warnings, {"You did not include a comment on the first line describing the use and args of your verb.", count}};
      endif
      "check if we have found the first real line of code or if it's just a comment";
      if (!first_real_line && !this:match_comment(line))
        first_real_line = count;
      endif
      "check for an argument scatter in a player facing verb";
      if (!internal && this:match_arg_scatter(line))
        warnings = {@warnings, {"You have used an argument scatter in a verb that is not 'this none this'. This is not how it should work.", count}};
      endif
      "find nesting";
      if (this:match_if(line))
        open_ifs = open_ifs + 1;
      elseif (this:match_for(line))
        open_fors = open_fors + 1;
      elseif (this:match_while(line))
        open_whiles = open_whiles + 1;
      elseif (this:match_endif(line))
        open_ifs = open_ifs - 1;
      elseif (this:match_endfor(line))
        open_fors = open_fors - 1;
      elseif (this:match_endwhile(line))
        open_whiles = open_whiles - 1;
      elseif (this:match_fork(line))
        forks = forks + 1;
      elseif (this:match_object(line))
        warnings = {@warnings, {"You may have an object number in your code. This should be avoided. Use corified references ($) instead.", count}};
      endif
      "check for tostr usage in :tell verbs";
      if (this:match_tell_tostr(line))
        warnings = {@warnings, {"tostr usage found inside a :tell call. This is uneeded as :tell will call tostr on all it's args.", count}};
      endif
      if (this:match_location_assignment(line))
        "this checks if you are doing like obj.location = something to make sure you aren't going to move an object(s) you don't mean to";
        warnings = {@warnings, {"IMPORTANT! You may have included a .location assignment INSTEAD of a comparison (= instead of ==)!", count}};
      endif
      if (this:match_if_assignment(line))
        warnings = {@warnings, {"You are doing an assignment '=' operation in an if statement, please confirm you didn't mean to do an equality check '=='.", count}};
      endif
      if (this:match_recycler_valid(line))
        warnings = {@warnings, {"You are doing an if($recycler:valid()) operation without a ! in front of it, are you SURE that you don't need the bang (!)?", count}};
      endif
      if ((current_nest = open_fors + open_ifs + open_whiles) > max_nest)
        max_nest = current_nest;
      endif
    endfor
    if (forks)
      warnings = {@warnings, {"There is a fork() in this code. Please do not do this unless you know what you are doing. Consider using something to schedule this verb to be run later instead.", 0}};
    endif
    if (max_nest > MAX_NESTING_WARNING)
      warnings = {@warnings, {tostr("Max nesting of if/for/while's is ", max_nest, ". Try refactoring or extracting pieces to a new verb to get your max nesting to 2 or below."), 0}};
    endif
    length_of_verb = length(code);
    if (length_of_verb > MAX_LENGTH_WARNING)
      warnings = {@warnings, {tostr("This verb is ", length_of_verb, " lines long. Consider refactoring or extracting to a new verb to get your max lines to ", MAX_LENGTH_WARNING, " or below."), 0}};
    endif
    return warnings;
  endverb

  verb "display_issues" (this none this) owner: #36 flags: "rxd"
    ":display_issues(LIST warnings) => none";
    "takes the output of :scan_for_issues and displays it";
    {warnings} = args;
    for warning_set in (warnings)
      {warning, line_number} = warning_set;
      if (line_number)
        player:tell($ansi:bryellow(tostr("Warning on line ", line_number)), ": ", warning);
      else
        player:tell($ansi:bryellow("Warning"), ": ", warning);
      endif
    endfor
  endverb

  verb "match_if" (this none this) owner: #36 flags: "rxd"
    ":match_if(STR line) => bool";
    {line} = args;
    return match(line, "^[ ]*if ");
  endverb

  verb "match_for" (this none this) owner: #36 flags: "rxd"
    ":match_for(STR line) => bool";
    {line} = args;
    return match(line, "^[ ]*for ");
  endverb

  verb "match_while" (this none this) owner: #36 flags: "rxd"
    ":match_while(STR line) => bool";
    {line} = args;
    return match(line, "^[ ]*while ");
  endverb

  verb "match_endif" (this none this) owner: #36 flags: "rxd"
    ":match_endif(STR line) => bool";
    {line} = args;
    return match(line, "^[ ]*endif$");
  endverb

  verb "match_endfor" (this none this) owner: #36 flags: "rxd"
    ":match_if(STR line) => bool";
    {line} = args;
    return match(line, "^[ ]*endfor$");
  endverb

  verb "match_endwhile" (this none this) owner: #36 flags: "rxd"
    ":match_endwhile(STR line) => bool";
    {line} = args;
    return match(line, "^[ ]*endwhile");
  endverb

  verb "match_fork" (this none this) owner: #36 flags: "rxd"
    ":match_fork(STR line) => bool";
    {line} = args;
    return match(line, "^[ ]*fork");
  endverb

  verb "match_object" (this none this) owner: #36 flags: "rxd"
    ":match_object(STR line) => bool";
    {line} = args;
    return match(line, "^#[0-9]+");
  endverb

  verb "match_tell_tostr" (this none this) owner: #36 flags: "rxd"
    ":match_tell_tostr(STR line) => bool";
    {line} = args;
    return match(line, "^.*:tell(.*tostr(");
  endverb

  verb "match_comment" (this none this) owner: #36 flags: "rxd"
    ":match_comment(STR line) => bool";
    {line} = args;
    return match(line, "^[ ]*\"");
  endverb

  verb "match_arg_scatter" (this none this) owner: #36 flags: "rxd"
    ":match_arg_scatter(STR line) => bool";
    {line} = args;
    return match(line, "{.+} = args;");
  endverb

  verb "match_location_assignment" (this none this) owner: #36 flags: "rxd"
    ":match_if(STR line) => bool";
    {line} = args;
    return match(line, "^[ ]*if (.+.location = .+)");
  endverb

  verb "match_if_assignment" (this none this) owner: #36 flags: "rxd"
    ":match_if_assignment(STR line) => bool";
    "looks for assignment operators in if statements";
    {line} = args;
    return match(line, "^[ ]*if (.+ = .+)");
  endverb

  verb "match_recycler_valid" (this none this) owner: #36 flags: "rxd"
    ":match_recycler_valid(STR line) => LIST";
    {line} = args;
    return match(line, "^[ ]*if (%$recycler:valid");
  endverb

endobject
