#!/usr/bin/env python3

"""a simple filter to tame commands that don't terminate their output with a newline"""

import sys
import os

if 'RLWRAP_FILTERDIR' in os.environ:
    sys.path.append(os.environ['RLWRAP_FILTERDIR'])
else:
    sys.path.append('.')

import rlwrapfilter

filter = rlwrapfilter.RlwrapFilter()
filter.help_text = ("Usage: rlwrap [-options] -z \"empty-line.py\" <command>\n" +
                    "removes all empty lines or lines made of entirely whitespace")


# substitute_prompt = sys.argv[1] if len(sys.argv) == 2 else " > "


def output_handler(last_line):
    # filter.send_output_oob(f"{last_line}\n")
    return last_line if last_line.strip() else ""


filter.output_handler = output_handler
filter.run()
