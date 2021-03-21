% ZIPUTIL(1) Common operations on files within a zip
%
% 2021-03-21


# NAME

ziputil - utility operations for zip contents

# SYNOPSIS

    ziputil list [-o -a] <zipfile> <query>...
    ziputil choose [-o -a] <zipfile> <query>...
    ziputil view [-o -a] <zipfile> <query>...

# COMMANDS

**list** - show all the files in a zip that have a filename matching *query*

**choose** - show all the files in a zip that have a filename matching *query*,
and then prompt for which of the enumeration to extract to the current folder.

**view** - show all the files in a zip that have a filename matching *query*,
and then prompt for which of the enumeration to display in the terminal
(assuming plaintext files).

# OPTIONS

**-o** **\-\-ordered**
:  order the files alphabetically

**-a** **\-\-any**
: match *any* query word, rather than *all* query words


# AUTHORS

Chris Davison <c.jr.davison@gmail.com>
