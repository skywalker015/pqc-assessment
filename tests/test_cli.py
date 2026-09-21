from pqc_assessment.cli import build_parser


def test_build_parser_defaults():
    parser = build_parser()
    args = parser.parse_args([])
    assert args.name == "world"


def test_build_parser_accepts_name():
    parser = build_parser()
    args = parser.parse_args(["--name", "Alice"])
    assert args.name == "Alice"
