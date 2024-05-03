# ezboard

This project aims to be a very simple replacement for tensorboard during prototyping.

*ezboard* parses the stdout of a training script for loss values. The lines have to contain the word "loss". Some variants are possible, e.g. the line "Epoch 5/100 Loss: 3.5 Acc: 80%" will be parsed as "3.5". More heuristics will be added in the future.

## Usage
Just pipe the output of your training script into ezboard. By default lines are buffered by python before sent to ezboard, which causes a delay. To disable line buffering you can pass the `-u` to the python interpreter:
```bash
python -u train.py | ezboard
```
Alternatively you can pass a logfile directly
```bash
ezboard train.log
```
Use `ezboard -h` for all command line options (e.g. smoothing).

There are some hotkeys that will be expanded in the future:
| Shortcut | Description|
|-----------|------------|
| <kbd>p</kbd><kbd>P</kbd> | Toggle between graph and raw log output |
| <kbd>q</kbd><kbd>Q</kbd> | Shutdown ezboard. This doesn't stop the training |
