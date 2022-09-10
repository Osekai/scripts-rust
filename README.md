# osekai-scripts

Command line executable to gather [osu!](https://osu.ppy.sh/) data like medals, users, and badges, process it, and upload it to [osekai](http://osekai.net/).

## Setup

- Acquire the executable file, either by downloading one from the [releases](https://github.com/Osekai/scripts-rust/releases) or clone the repository and compile it yourself with `cargo build --release` (requires [Rust](https://www.rust-lang.org/) to be installed)
- Make sure you have a `.env` file next to the executable. You should use the `.env.example` file contained in this repository and fill in proper values

## How it works

The script works with tasks so that it either runs one single task and finishes or it runs a list of tasks (schedule) over and over without stopping.

A task consists of the following flags:
- `medal`: Current medals will be retrieved and uploaded
- `rarity`: Next to osekai users, also retrieve all leaderboard users, then calculate medal rarity and upload it
- `ranking`: Process osekai users (and leaderboard users if `rarity` is set) and upload their ranking data
- `badge`: For all available users, process their badges and upload them

When specifying tasks, do so with a `|`-separated list of these flags.
You can also use these predefined tasks:
- `default`: `medal | ranking | badge`
- `full`: `medal | ranking | badge | rarity`

In case the script runs a schedule, there will be an interval between two executing tasks e.g. if the interval is 12 hours and the first task takes 2 hours then the next task will start 10 hours after the first task ended. If a task takes longer than the specified interval then there is no wait time inbetween tasks.

## Arguments

- `--extra` (`-e`): Specify a user id that should be included in tasks. This can be added multiple this.
- `--help` (`-h`): Show help text.
- `--interval` (`-i`): Specify the time in hours inbetween two tasks. Defaults to 12 hours.
- `--initial-delay`: Specify the time in minutes that should be waited before starting the first task. Defaults to 1 minute when looping or 0 minutes when running one task.
- `--progress` (`-p`): While requesting user data, send progress info to osekai.
- `--quiet` (`-q`): Don't show any logs.
- `--task` (`-t`): Run only this one task instead of running a schedule in a loop. This can be added multiple times to build a task consisting of multiple flags.

## Examples

```sh
osekai-script -e 42 --initial-delay 2 --interval 7 --extra 2211396 -p
```
After an initial delay of 2 minutes this will run the schedule specified in the `.env` file with an interval of 7 hours inbetween tasks. Each task will consider the two additional user ids 2211396 and 42. Whenever users are requested, progress info will be sent to osekai.

```sh
osekai-script -i 7 -t medal -q --task "badge | rarity" -e 2
```
This will run only one task so the 7 hours of interval are redundant. The task consists of medals, badges, and medal rarity. The user id 2 is certain to be considered in the task. No logs will be displayed.
