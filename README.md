# Secure Vault Personal Information (SVPI)

## Project Description

Secure Vault Personal Information (SVPI) is software that securely stores personal data on a [Blaustahl Storage Device](https://github.com/binqbit/blaustahl). The project is developed based on a [Simple Read Write Protocol (SRWP)](https://github.com/binqbit/serialport_srwp), ensuring reliable interaction with the device through a serial port. SVPI provides users with functionality for managing, organizing, and optimizing data storage, making the use of the Blaustahl device more convenient and efficient.

The primary goal of SVPI is to offer a simple and intuitive interface for working with personal data, which avoids complex operations and eases data management for the user. The project is targeted at those who need a reliable and easily accessible storage solution for confidential information.

## Build

To build the SVPI project, execute the provided build scripts for your operating system: [Linux](./build.sh) and [Windows](./build.bat).

## Commands

SVPI supports a number of commands that help users interact with the Blaustahl Storage Device:

- `svpi init / i <memory_size> [low|medium|strong]`: Initialize the device for the desired architecture. This command prepares the device for operation by creating the necessary data architecture.

- `svpi check / c`: Check the status of the device. Useful for verifying the device's connection and current state.

- `svpi format / f`: Format the data in the device. Allows the user to clear all saved data if needed.

- `svpi optimize / o`: Optimize the memory. Combines free space and removes fragmentation to make more space available for new data.

- `svpi export / e <file_name>`: Export data to a file. Allows the user to save data from the device to an external file.

- `svpi import / m <file_name>`: Import data from a file. Allows the user to load data from an external file to the device.

- `svpi dump / d <file_name>`: Dump the data from the device to a file. This command allows the user to save all data from the device to an external file for backup or analysis.

- `svpi load / ld <file_name>`: Load dump data from a file to the device. This command allows the user to restore data from an external file to the device.

- `svpi set-master-password / set-master`: Set master password. Allows the user to set a master password for device encryption.

- `svpi reset-master-password / reset-master`: Reset master password. Allows the user to reset the master password for device encryption.

- `svpi check-master-password / check-master`: Check master password. Allows the user to verify the master password.

- `svpi add-encryption-key / add-key <name>`: Add encryption key. Allows the user to add a new encryption key for data protection.

- `svpi list / l`: Print all data list. Displays a list of all saved data for quick viewing of available information.

- `svpi set / s <name> <data>`: Set data. This command allows the user to enter new information into the storage.

- `svpi get / g <name>`: Get data. Retrieves data by the specified name for easy access to specific information.

- `svpi remove / r <name>`: Remove data. Deletes data by the specified name to free up memory.

- `svpi rename / rn <old_name> <new_name>`: Rename data. Allows the user to change the name of existing data.

- `svpi change-data-type / cdt <name> <new_data_type>`: Change data type. Allows the user to modify the data type of existing entries.

- `svpi change-password / cp <name>`: Change data password. To remove encryption, omit the new password.

- `svpi version / v`: Print the version of the application. Useful for checking the software version and ensuring it is up to date.

- `svpi help / h`: Print this help message. Displays a list of available commands and their descriptions.

- `svpi --mode=server`: Start the API server. Allows developers to integrate SVPI into their software.

- `svpi --mode=chrome`: Start the Chrome app. Allows users to interact with the device through a Chrome extension.

## Flags

- `svpi --mode=<cli|json|server|chrome>`: Select application mode (default: `cli`).

- `svpi <command> --confirm`: Confirm destructive actions (required in `--mode=json`).

- `svpi get --clipboard / -c`: Copy data to clipboard. Automatically copies retrieved data to the system clipboard.

- `svpi get <name> --password=<password>`: Provide password via command line. Allows specifying the password directly in the command.

- `svpi set <name> <value> --password=<password>`: Provide password via command line. Allows specifying the password directly when setting data.

- `svpi change-password <name> --old-password=<old_password> --new-password=<new_password>`: Provide old and new passwords via command line. Allows changing passwords without interactive prompts.

- `svpi change-password <name> --old-password=<old_password>`: Remove encryption (provide only `--old-password`). Decrypts an entry and saves it unencrypted.

- `svpi --mode=server --auto-exit`: Automatically exit the API server after device disconnection. Ensures the server closes when the device is no longer available.

- `svpi <command> --file=<file_name>`: Open a file password storage. Allows working with file-based password storage instead of device storage.

## Export/Import Format

### Text file list of data with the following format

- Plain data:

```plaintext
<name> = <data>
```

- Encrypted data:

```plaintext
<name> = data:application/vnd.binqbit.svpi;fp=<password_fingerprint>;<data_type>,<hex_data>
```

### How can this be used?

This can be useful for transferring data between devices, backing up, or securely migrating data between software versions.

## Master Password Encryption

Master Password is the main root password for restoring data. This password generates encryption keys that encode data. These encryption keys are encoded with simple passwords that are entered from the keyboard. But in the case of a loss of passwords, everything can be restored using the Master Password.

### Decrypt All Data And Encrypt With New Password

```shell
svpi export <file_name>
svpi import <file_name>
rm <file_name>
```

### How it Works?

The master password is used to encrypt data and manage access to the device. Encryption keys can be added for additional layers of security for specific data entries.

### Why This is Needed?

You can use the master password and encryption keys for securing data without worrying about accidentally revealing them. If someone gains access to your encrypted data, they won't be able to decrypt it without the proper passwords and keys.

### Note

- If you suspect that your data has been compromised, change the master password and encryption keys to protect your data.
- If you forget the master password, you can reset it, but this may result in loss of encrypted data.
- Since data encryption depends on passwords and keys, losing them may result in permanent data loss.

## Data Storage Architecture

SVPI uses a carefully designed segment architecture for managing and storing data on the Blaustahl device. This structure allows for efficient organization of information, ensuring quick access and ease of management.

### Data Storage Format

1. Metadata Initialization:

   - `"\0<METADATA>\0"`: Marker for the start of metadata segment initialization.
   - `<version> (4 bytes)`: Four bytes allocated to store the architecture version.
   - `<memory size> (4 bytes)`: Four bytes allocated to store the size of the device's entire memory.
   - `<dump protection> (1 byte)`: Global dump protection level (multiplier: low=1, medium=2, strong=4).
   - `<master password hash> (32 bytes)`: A 32-byte hash of the master password for data encryption.
   - `"\0</METADATA>\0"`: Marker for the end of metadata segment initialization.

2. Segment Data:

   - `<segment 1..N bytes...> (space specified in metadata)`: A sequence of segment data from the first to the Nth, each storing information saved by the user.

3. Segment Metadata:
   - `<segment metadata N..1>`: Sequence of segment metadata starting from the last segment and ending with the first. Each metadata entry includes:
     - Address (4 bytes): Indicates the starting position of the segment in memory.
     - Size (4 bytes): Determines the amount of memory occupied by the segment.
     - Data Type (1 byte): Specifies the type of data:
       - 0: Plain
       - 1: Encrypted
     - Status (1 byte): Indicates the status of the segment:
       - 0: Deleted
       - 1: Active
     - Name (32 bytes): A string identifying the segment allowing the user to easily locate and manage it.
   - `<number of segments> (4 bytes):` A number indicating the total number of segments stored on the device. Located directly before the segment metadata and occupies 4 bytes.

### Why This Structure is Needed?

This architecture provides a clear organization of data on the device, allowing for easy addition, extraction, and deletion of information. Initialization markers help verify data integrity, while segment metadata ensures ease of management and memory optimization. This storage method effectively utilizes available space and minimizes fragmentation, thereby increasing the device's performance.

### API

[API](./api/api.md) for developers who want to integrate SVPI into their software.
