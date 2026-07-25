#!/usr/bin/env python3
"""
Python script to automate the systemd user service and timer setup
for the wallswitch application with a customizable rotation interval.
"""

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


class SystemdConfigurator:
    """Manages systemd user configuration and binary environment checks for wallswitch."""

    def __init__(self, interval_seconds: int):
        self.interval_seconds = interval_seconds
        self.home = Path.home()

        # Define paths securely using Pathlib
        self.cargo_bin_dir = self.home / ".cargo" / "bin"
        self.cargo_bin = self.cargo_bin_dir / "wallswitch"
        self.systemd_user_dir = self.home / ".config" / "systemd" / "user"
        self.service_path = self.systemd_user_dir / "wallswitch.service"
        self.timer_path = self.systemd_user_dir / "wallswitch.timer"

    @property
    def description_time(self) -> str:
        """Format the time interval dynamically for readability."""
        if self.interval_seconds % 60 == 0:
            return f"{self.interval_seconds // 60} minutes"
        return f"{self.interval_seconds} seconds"

    def verify_environment(self) -> None:
        """Verify that essential external tools are available in the system."""
        if not shutil.which("systemctl"):
            raise RuntimeError(
                "The 'systemctl' executable was not found. Systemd is required."
            )

    def ensure_directories(self) -> None:
        """Verify and safely create required directories if they do not exist."""
        try:
            # Create the cargo binary path if it is missing
            if not self.cargo_bin_dir.exists():
                print(f"Creating cargo binary directory: {self.cargo_bin_dir}")
                self.cargo_bin_dir.mkdir(parents=True, exist_ok=True)

            # Ensure systemd user directory exists with restricted user permissions (rwx------)
            if not self.systemd_user_dir.exists():
                print(f"Creating systemd user directory: {self.systemd_user_dir}")
                self.systemd_user_dir.mkdir(parents=True, exist_ok=True)
                self.systemd_user_dir.chmod(0o700)

        except OSError as err:
            raise OSError(f"Failed to create or set directory permissions: {err}")

    def verify_binary(self) -> None:
        """Check if the wallswitch binary is ready and executable."""
        if not self.cargo_bin.exists():
            print(
                f"Warning: Binary not found at {self.cargo_bin}",
                file=sys.stderr,
            )
            print(
                "Please run 'cargo install --path=.' to build and place the binary.",
                file=sys.stderr,
            )
        elif not os.access(self.cargo_bin, os.X_OK):
            print(
                f"Warning: Binary at {self.cargo_bin} exists but is not executable.",
                file=sys.stderr,
            )

    def write_unit_files(self) -> None:
        """Generate and write service and timer files with secure file permissions."""
        service_content = f"""[Unit]
Description=Wallswitch Wallpaper Rotator
After=graphical-session.target

[Service]
Type=oneshot
ExecStart={self.cargo_bin.resolve()} --once
"""

        timer_content = f"""[Unit]
Description=Run Wallswitch every {self.description_time}

[Timer]
OnActiveSec=0sec
OnUnitActiveSec={self.interval_seconds}sec
AccuracySec=1s

[Install]
WantedBy=timers.target
"""

        try:
            # Write systemd service file
            self.service_path.write_text(service_content, encoding="utf-8")
            self.service_path.chmod(0o600)  # Owner read/write only (rw-------)
            print(f"Service file written to: {self.service_path}")

            # Write systemd timer file
            self.timer_path.write_text(timer_content, encoding="utf-8")
            self.timer_path.chmod(0o600)  # Owner read/write only (rw-------)
            print(f"Timer file written to: {self.timer_path}")

        except OSError as err:
            raise OSError(f"Failed to write systemd unit files: {err}")

    def reload_and_enable(self) -> None:
        """Interact with systemctl to reload configurations and enable the timer."""
        print("Reloading systemd user daemon...")
        subprocess.run(
            ["systemctl", "--user", "daemon-reload"],
            check=True,
            capture_output=True,
            text=True,
        )

        print("Enabling and starting wallswitch.timer...")
        subprocess.run(
            ["systemctl", "--user", "enable", "--now", "wallswitch.timer"],
            check=True,
            capture_output=True,
            text=True,
        )

    def print_diagnostic_commands(self) -> None:
        """Display recommended terminal commands to monitor and manage the services."""
        print("\n" + "=" * 65)
        print("Systemd Service Diagnostic Commands:")
        print("=" * 65)
        print("Check active timers:")
        print("  systemctl --user list-timers")
        print("\nCheck current timer status:")
        print("  systemctl --user status wallswitch.timer")
        print("\nCheck service configuration and execution status:")
        print("  systemctl --user status wallswitch.service")
        print("\nInspect logs and output associated with the service:")
        print("  journalctl --user -u wallswitch.service")
        print("=" * 65)


def main():
    # Setup Argument Parser
    parser = argparse.ArgumentParser(
        description="Configure systemd user service and timer for wallswitch safely."
    )
    parser.add_argument(
        "-s",
        "--seconds",
        type=int,
        default=300,
        help="Interval in seconds between wallpaper rotations (default: 300 / 5 minutes).",
    )
    args = parser.parse_args()

    # Boundary check to prevent resource exhaustion or systemd spam
    MIN_INTERVAL_SECONDS = 5
    if args.seconds < MIN_INTERVAL_SECONDS:
        print(
            f"Error: The interval must be at least {MIN_INTERVAL_SECONDS} seconds.",
            file=sys.stderr,
        )
        sys.exit(1)

    # Initialize configurator instance
    configurator = SystemdConfigurator(args.seconds)

    try:
        configurator.verify_environment()
        configurator.ensure_directories()
        configurator.verify_binary()
        configurator.write_unit_files()
        configurator.reload_and_enable()

        print(
            f"\nSetup completed successfully! Wallswitch will run every {configurator.description_time}."
        )

        configurator.print_diagnostic_commands()

    except RuntimeError as err:
        print(f"\nEnvironment error encountered: {err}", file=sys.stderr)
        sys.exit(1)
    except subprocess.CalledProcessError as err:
        print(
            f"\nError executing systemd commands: {err.stderr.strip() if err.stderr else err}",
            file=sys.stderr,
        )
        sys.exit(1)
    except OSError as err:
        print(f"\nFile system I/O error: {err}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
