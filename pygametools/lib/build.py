from enum import Enum

import os


class BuildMode(Enum):
    """Specifies the build mode (or type) that is generated.

    `BuildMode.Dev` is a development mode build with symbols and optimization off.
    `BuildMode.Release` is an optimized build.
    """

    Dev = 1
    Release = 2

    @staticmethod
    def from_str(s: str) -> "BuildMode":
        """Converts a string to a `BuildMode` enum value, or throws an exception if the value cannot be parsed."""
        if s == "dev" or s == "Dev":
            return BuildMode.Dev
        elif s == "release" or s == "Release":
            return BuildMode.Release
        else:
            raise Exception(f"invalid BuildMode value '{s}'")


class WebBuildConfig:
    """Holds a collection of configuration values relevant to building of the game."""

    def __init__(self, mode: BuildMode, verbose: bool, out_dir: str, pkg_dir: str):
        """
        out_dir: Directory where the build output will be copied to.
        pkg_dir: Name of directory relative to `out_dir` where WASM binary is copied to.
        """

        self.mode = mode
        self.verbose = verbose
        self.output_dir = out_dir
        self.pkg_dir = os.path.join(self.output_dir, pkg_dir)

    def content_dir(self):
        return "content"
