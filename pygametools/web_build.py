#!/usr/bin/env python3
from .lib.build import BuildMode, WebBuildConfig
from .lib.webserver import HttpServer

import argparse
import logging
import os
import shutil
import subprocess
import threading


# TODO: Warn if wasm-pack not installed
# TODO: Timing benchmarks


def build(build_config: WebBuildConfig):
    build_args = ["build"]
    build_args += ["--target", "web"]
    build_args += ["--out-dir", build_config.pkg_dir]

    if build_config.mode == BuildMode.Dev:
        build_args += ["--dev"]
    else:
        build_args += ["--release"]

    logging.info(build_args)

    logging.info("START BUILD")
    result = subprocess.run(["wasm-pack"] + build_args)

    if result.returncode != 0:
        logging.error("BUILD FINISHED WITH ERRORS")
        return False
    else:
        logging.info("BUILD OK")
        return True


def package(build_config: WebBuildConfig):
    logging.info("START PACKAGING")

    output_content_dir = os.path.join(build_config.output_dir, "content")
    logging.debug(f"output content dir path = {output_content_dir}")

    # Create the output directory if it does not already exist. Make sure the
    # application's index.html that starts everything up is also copied to this
    # output directory.
    if not os.path.isdir(build_config.output_dir):
        logging.debug(f"CREATE {build_config.output_dir}")
        os.mkdir(build_config.output_dir)

    logging.debug("COPY: index.html")
    shutil.copy("index.html", os.path.join(build_config.output_dir, "index.html"))

    # Copy game content to the output content directory.
    if not os.path.isdir(output_content_dir):
        logging.debug(f"CREATE {output_content_dir}")
        os.mkdir(output_content_dir)

    for src_dir, dirs, files in os.walk(build_config.content_dir()):
        dest_dir = src_dir.replace(build_config.content_dir(), output_content_dir, 1)
        if not os.path.exists(dest_dir):
            os.makedirs(dest_dir)
        for f in files:
            src_file = os.path.join(src_dir, f)
            dest_file = os.path.join(dest_dir, f)

            if os.path.exists(dest_file):
                logging.debug(f"DELETE {dest_file}")
                os.remove(dest_file)

            logging.debug(f"COPY {src_file} --> {dest_dir}/{src_file}")
            shutil.copy(src_file, dest_dir)

    # Packaging complete.
    logging.info("PACKAGING OK")


def main():
    # Argument parsing.
    parser = argparse.ArgumentParser()

    parser.add_argument("-o", "--out_dir", default="webpkg")
    parser.add_argument("-m", "--mode", choices=["dev", "release"], default="dev")
    parser.add_argument("-i", "--interactive", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    parser.add_argument(
        "--skip-build", action="store_true", default=False, help="Build Rust code"
    )
    parser.add_argument(
        "--skip-package", action="store_true", default=False, help="Package game"
    )
    parser.add_argument("-p", "--http_port", default=9000)
    parser.add_argument("--pkg-dir", default="pkg")

    args = parser.parse_args()

    # Use verbose logging?
    logging.basicConfig()

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    # Build config.
    build_config = WebBuildConfig(
        mode=BuildMode.from_str(args.mode),
        verbose=args.verbose,
        out_dir=args.out_dir,
        pkg_dir=args.pkg_dir,
    )

    # Run build pipeline.
    if not args.skip_build:
        if not build(build_config):
            logging.error("Aborting because build failed! :(")
            return False

    if not args.skip_package:
        package(build_config)

    # Interactive loop - keep running and ask for user commands.
    if args.interactive:
        httpd = HttpServer(int(args.http_port), build_config.output_dir)
        httpd_thread = threading.Thread(
            target=lambda: httpd.run(),
        )

        httpd_thread.start()

        print("Commands: (q)uit, (b)uild, (p)ackage")
        keep_running = True

        while keep_running:
            command = input("> ")

            if command == "q" or command == "quit":
                logging.info("quit command received")
                httpd.stop()
                keep_running = False
            elif command == "b" or command == "build":
                build(build_config)
            elif command == "p" or command == "package":
                package(build_config)
            else:
                print("unknown command")

        httpd_thread.join()


if __name__ == "__main__":
    main()
    print("done!")
