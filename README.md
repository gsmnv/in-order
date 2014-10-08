# In order

Do whatever you want incrementally in specific order(think DB
migrations). See `examples` for sample configuration file.

## Usage

To run actions from last saved index:

    in-order do

To undo all made actions from last saved index:

    in-order undo

By default `in-order` will search for configuration file(`do.toml`) in
current directory, to specify your own path to configuration file use
`-c`:

    in-order do -c examples/migrations/do.toml

And of course you can bring up usage instructions with:

    in-order -h

## Features

- Sequential execution on 'do' files.

- Reverse execution of 'undo' files.

- Special commands for specific actions.

- Execution is stopped and last successful action's index is saved if
  command fails.
