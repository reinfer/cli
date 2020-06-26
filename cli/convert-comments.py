#!/usr/bin/env python
import fileinput
import json
import logging
import sys
from argparse import ArgumentParser
from argparse import Namespace as ArgumentNamespace
from typing import Any, Dict, NewType

log = logging.getLogger(__name__)

CommentDict = NewType("CommentDict", Dict[str, Any])


def convert_comment(old_comment: CommentDict) -> CommentDict:
    """
    Converts comments with annotations from the old format (flat) to the one
    used by the CLI with top level keys: `commment`, `labellings`, `entities`.
    """
    assigned_labels = old_comment.get("assigned_labels")
    dismissed_labels = old_comment.get("dismissed_labels")
    assigned_entities = old_comment.get("assigned_entities")
    dismissed_entities = old_comment.get("dismissed_entities")

    labellings = {}
    if assigned_labels is not None:
        labellings["assigned"] = assigned_labels
        del old_comment["assigned_labels"]
    if dismissed_labels is not None:
        labellings["dismissed"] = dismissed_labels
        del old_comment["dismissed_labels"]

    entities = {}
    if assigned_entities is not None:
        entities["assigned"] = assigned_entities
        del old_comment["assigned_entities"]
    if dismissed_entities is not None:
        entities["dismissed"] = dismissed_entities
        del old_comment["dismissed_entities"]

    new_comment = CommentDict({"comment": old_comment})
    if len(labellings) > 0:
        new_comment["labelling"] = labellings
    if len(entities) > 0:
        new_comment["entities"] = entities

    return new_comment


def run(args: ArgumentNamespace) -> None:
    new_comments = map(
        convert_comment, map(json.loads, fileinput.input(args.file))
    )
    for comment_str in map(json.dumps, new_comments):
        sys.stdout.write(comment_str)
        sys.stdout.write("\n")
    sys.stdout.flush()


def configure_parser(parser: ArgumentParser) -> None:
    parser.description = """
    Converts comments with annotations from the old format (flat) to the one
    used by the CLI with top level keys: `commment`, `labellings`, `entities`.
    """
    parser.add_argument(
        "file",
        type=str,
        action="store",
        metavar="PATH",
        help=(
            "Path to a jsonl file with comments in old format. "
            "Pass - to use stdin"
        ),
    )


def main() -> None:
    parser = ArgumentParser()
    configure_parser(parser)
    args = parser.parse_args()
    run(args)


if __name__ == "__main__":
    main()
