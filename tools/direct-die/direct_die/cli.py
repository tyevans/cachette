"""The command line interface of direct-die."""

from __future__ import annotations

import argparse
import sys

from . import guide as guide_module
from . import loop, render, session
from .client import BASE_URL, MODEL, ClientError


def _run(arguments: argparse.Namespace) -> int:
    """Run one session and report the score of each round."""
    try:
        results = loop.run_session(
            asset=arguments.asset,
            subject=arguments.subject,
            rounds=arguments.rounds,
            variants=arguments.variants,
            session_id=arguments.session,
            exemplar_limit=arguments.exemplars,
        )
    except guide_module.GuideError as error:
        print(f"guide error: {error}", file=sys.stderr)
        return 2
    except ClientError as error:
        print(f"endpoint error: {error}", file=sys.stderr)
        return 3

    print()
    print("round  a    b    c    d    best  seconds  tokens")
    for result in results:
        cells = []
        for letter in session.VARIANT_LETTERS:
            found = [item for item in result.variants if item.letter == letter]
            score = found[0].score if found else None
            cells.append("-" if score is None else str(score))
        best = result.best()
        print(
            f"{result.index:<6} "
            + " ".join(f"{cell:<4}" for cell in cells)
            + f" {('-' if not best else best.letter + ' ' + str(best.score)):<5} "
            f"{result.seconds:<8.1f} "
            f"{result.prompt_tokens + result.completion_tokens}"
        )
    return 0


def _guide(arguments: argparse.Namespace) -> int:
    """Report what the guide holds."""
    for asset in guide_module.asset_types():
        try:
            loaded = guide_module.load(asset)
        except guide_module.GuideError as error:
            print(f"{asset}: {error}")
            continue
        sizes = render.sizes_for(asset)
        names = ", ".join(loaded.exemplar_names) or "none"
        print(
            f"{asset}: version {loaded.version}, display {sizes.display} px, "
            f"inspection {sizes.inspection} px, exemplars: {names}"
        )
    return 0


def _probe(arguments: argparse.Namespace) -> int:
    """Check that the endpoint answers, and report the cost."""
    from .client import Image, ask_with_images

    exemplar = render.render(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">'
        '<circle cx="32" cy="32" r="24" fill="#c04040"/></svg>',
        256,
    )
    try:
        reply = ask_with_images(
            "Name the shape and the colour in the image. Answer in five "
            "words or fewer.",
            [Image(label="TEST IMAGE", data=exemplar)],
            max_tokens=60,
        )
    except ClientError as error:
        print(f"endpoint error: {error}", file=sys.stderr)
        return 3
    print(f"endpoint {BASE_URL}, model {MODEL}")
    print(f"answer: {reply.text.strip()[:200]}")
    print(
        f"{reply.seconds:.1f} s, {reply.prompt_tokens} prompt tokens, "
        f"{reply.completion_tokens} completion tokens"
    )
    return 0


def main(argv: list[str] | None = None) -> int:
    """Run the command line interface."""
    parser = argparse.ArgumentParser(
        prog="direct-die",
        description=(
            "Draw a game asset as SVG, render it, and let a vision model "
            "critique and revise it on a loop."
        ),
    )
    commands = parser.add_subparsers(dest="command", required=True)

    run = commands.add_parser("run", help="run a refine session")
    run.add_argument("--asset", default="hex-tile", help="the asset type")
    run.add_argument(
        "--subject",
        required=True,
        help="what the asset shows, for example 'a dense pine forest'",
    )
    run.add_argument("--rounds", type=int, default=3, help="how many rounds")
    run.add_argument(
        "--variants", type=int, default=4, choices=[1, 2, 3, 4],
        help="how many variants in each round",
    )
    run.add_argument(
        "--session",
        default=None,
        help="continue an existing session identifier",
    )
    run.add_argument(
        "--exemplars",
        type=int,
        default=guide_module.DEFAULT_EXEMPLAR_LIMIT,
        help="how many exemplar images to attach to a critique",
    )
    run.set_defaults(handler=_run)

    commands.add_parser("guide", help="report what the guide holds").set_defaults(
        handler=_guide
    )
    commands.add_parser("probe", help="check the endpoint").set_defaults(
        handler=_probe
    )

    arguments = parser.parse_args(argv)
    return arguments.handler(arguments)


if __name__ == "__main__":
    raise SystemExit(main())
