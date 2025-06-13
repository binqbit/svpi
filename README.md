# Secure Vault Personal Information (SVPI)

## Project Description

Secure Vault Personal Information (SVPI) is software that securely stores personal data on a [Blaustahl Storage Device](https://github.com/binqbit/blaustahl). The project is developed based on a [Simple Read Write Protocol (SRWP)](https://github.com/binqbit/serialport_srwp), ensuring reliable interaction with the device through a serial port. SVPI provides users with functionality for managing, organizing, and optimizing data storage, making the use of the Blaustahl device more convenient and efficient.

The primary goal of SVPI is to offer a simple and intuitive interface for working with personal data, which avoids complex operations and eases data management for the user. The project is targeted at those who need a reliable and easily accessible storage solution for confidential information.

## Build

To build the SVPI project, execute the provided build scripts for your operating system: [Linux](./build.sh) and [Windows](./build.bat).

## Commands

SVPI supports a number of commands that help users interact with the Blaustahl Storage Device:

- `svpi init / i <memory_size>`: Initialize the device for the desired architecture. This command prepares the device for operation by creating the necessary data architecture.

- `svpi format / f`: Formats all data on the device. Allows the user to clear all saved data if needed.

- `svpi dump / d <file_name>`: Dump the data from the device to a file. This command allows the user to save all data from the device to an external file for backup or analysis.

- `svpi load / ld <file_name>`: Load dump data from a file to the device. This command allows the user to restore data from an external file to the device.

- `svpi list / l`: Displays a list of all saved data. Useful for quickly viewing available information.

- `svpi set / s <name> <data>`: Saves data on the device. This command allows the user to enter new information into the storage.

- `svpi get / g <name>`: Retrieves data by the specified name. Simplifies access to specific information in the storage.

- `svpi remove / r <name>`: Deletes data by the specified name. Allows the user to free up memory by removing unnecessary data.

- `svpi optimize / o`: Optimizes memory usage. Combines free space and removes fragmentation to make more space available for new data.

- `svpi export / e [--password / -p] <file_name>`: Export data to a file with decryption option. Allows the user to save data from the device to an external file.

- `svpi import / m [--password / -p] <file_name>`: Import data from a file with encryption option. Allows the user to load data from an external file to the device.

- `svpi set-password / sp`: Set root password. Allows the user to set a password 2-level encryption for the device.

- `svpi reset-password / rp`: Reset root password. Allows the user to reset the password 2-level encryption for the device.

- `svpi get-password / gp`: Get root password. Allows the user to get the root password, if you forget the root password.

- `svpi check`: Check if the device supports SRWP protocol. Useful for verifying the device's compatibility with the SVPI software.

- `svpi version / v`: Displays the current version of the SVPI software. Useful for checking the software version and ensuring it is up to date.

- `svpi` or `svpi help / h`: Displays a list of available commands and their descriptions. Useful for quickly checking available functionality.

- `svpi api-server`: Start the API server. Allows developers to integrate SVPI into their software.

- `svpi api-chrome`: Start the Chrome app. Allows users to interact with the device through a Chrome extension.

## Flags

- `svpi <command> [flags...] [params...]`: How to use flags.

- `svpi list/set --password / -p`: Use password for encryption/decryption.

- `svpi set --password2 / -p2`: Use password with confirmation for encryption.

- `svpi set --root-encrypt / -re`: Use root password for encryption/decryption.

- `svpi set/get --clipboard / -c`: Copy data to/from clipboard.

- `svpi --view / -v`: View the data in the terminal.

- `svpi api-server --auto-exit / -ae`: Automatically exit the API server after device disconnection.

## Export/Import Format

### Text file list of data with the following format

- Plain data:
```plaintext
<name>: <data>
```

- Encrypted data:
```plaintext
@<name>: base64(encrypted(data))
```

### How can this be used?

This can be useful for transferring data between devices, backing up, or securely migrating data between software versions.

## Root Password Encryption

The root password is stored on the device in an encrypted form using the main password, allowing it to be used for encrypting data without concerns about leakage.

### Decrypt All Data And Encrypt With New Password

```shell
# [-re] - if you want to use the root password for encryption/decryption
svpi export -p [-re] <file_name>
svpi import -p [-re] <file_name>
rm <file_name>
```

### How it Works?

The root password is used to encrypt data, while the main password encrypts the root password. The root password can be long and inconvenient to enter but reliable, whereas the main password can be short and simple for everyday use.

### Why This is Needed?

You can use the root password for encrypting data without worrying about accidentally revealing it and compromising your data. For example, if someone steals your encrypted data, like a backup, they won't be able to decrypt it even if they discover your main password, because the data is encrypted with the root password. If you've misplaced the main password used to encrypt the root password, you can safely change the main password without worrying about the security of your data. Additionally, you don't need to worry if you lose the device, as the root password is encrypted with the main password.

### Note

- If you suspect that your data has been copied from the device, change the root and main passwords to protect your data.
- If you forget the main password, you can reset it by overwriting the root password with a new main password.
- Since all data is encrypted using the root password, if it is lost, you risk losing all data.

## Data Storage Architecture

SVPI uses a carefully designed segment architecture for managing and storing data on the Blaustahl device. This structure allows for efficient organization of information, ensuring quick access and ease of management.

### Data Storage Format

1. Data Initialization (INIT_SEGMENTS_DATA):
   - `"\0<INIT_SEGMENTS_DATA>\0"`: Marker for the start of segment initialization data.
   - `<version> (4 bytes)`: Four bytes allocated to store the architecture version.
   - `<memory size> (4 bytes)`: Four bytes allocated to store the size of the device's entire memory.
   - `<root password> (128 bytes)`: 128 bytes allocated to store the root password for the device.
   - `"\0</INIT_SEGMENTS_DATA>\0"`: Marker for the end of segment initialization data.

2. Segment Data:
   - `<segment 1..N bytes...> (space specified in metadata)`: A sequence of segment data from the first to the Nth, each storing information saved by the user.

3. Segment Metadata:
   - `<segment metadata N..1> (42 bytes)`: Sequence of segment metadata starting from the last segment and ending with the first. Each metadata entry includes:
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