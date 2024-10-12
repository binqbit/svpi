# Secure Vault Personal Information (SVPI)

## Project Description

Secure Vault Personal Information (SVPI) is software that securely stores personal data on a [Blaustahl Storage Device](https://github.com/binqbit/blaustahl). The project is developed based on a [Simple Read Write Protocol (SRWP)](https://github.com/binqbit/serialport_srwp), ensuring reliable interaction with the device through a serial port. SVPI provides users with functionality for managing, organizing, and optimizing data storage, making the use of the Blaustahl device more convenient and efficient.

The primary goal of SVPI is to offer a simple and intuitive interface for working with personal data, which avoids complex operations and eases data management for the user. The project is targeted at those who need a reliable and easily accessible storage solution for confidential information.

## Commands

SVPI supports a number of commands that help users interact with the Blaustahl Storage Device:

- `svpi init / i <memory_size>`: Initializes the device with the specified memory size. This command prepares the device for operation by creating the necessary data architecture.

- `svpi format / f`: Formats all data on the device. Allows the user to clear all saved data if needed.

- `svpi list / l`: Displays a list of all saved data. Useful for quickly viewing available information.

- `svpi set / s <name> <data>`: Saves data on the device. This command allows the user to enter new information into the storage.

- `svpi get / g <name>`: Retrieves data by the specified name. Simplifies access to specific information in the storage.

- `svpi remove / r <name>`: Deletes data by the specified name. Allows the user to free up memory by removing unnecessary data.

- `svpi optimize / o`: Optimizes memory usage. Combines free space and removes fragmentation to make more space available for new data.

- `svpi version / v`: Displays the current version of the SVPI software. Useful for checking the software version and ensuring it is up to date.

- `svpi` or `svpi help / h`: Displays a list of available commands and their descriptions. Useful for quickly checking available functionality.

## Data Storage Architecture

SVPI uses a carefully designed segment architecture for managing and storing data on the Blaustahl device. This structure allows for efficient organization of information, ensuring quick access and ease of management.

### Data Storage Format:

1. Data Initialization (INIT_SEGMENTS_DATA):
   - `"\0<INIT_SEGMENTS_DATA>\0"`: Marker for the start of segment initialization data.
   - `<memory size> (4 bytes)`: Four bytes allocated to store the size of the device's entire memory.
   - `"\0</INIT_SEGMENTS_DATA>\0"`: Marker for the end of segment initialization data.

2. Segment Data:
   - `<segment 1..N bytes...> (space specified in metadata)`: A sequence of segment data from the first to the Nth, each storing information saved by the user.

3. Segment Metadata:
   - `<segment metadata N..1> (40 bytes)`: Sequence of segment metadata starting from the last segment and ending with the first. Each metadata entry includes:
     - Address (4 bytes): Indicates the starting position of the segment in memory.
     - Size (4 bytes): Determines the amount of memory occupied by the segment.
     - Name (32 bytes): A string identifying the segment allowing the user to easily locate and manage it.
   - `<number of segments> (4 bytes):` A number indicating the total number of segments stored on the device. Located directly before the segment metadata and occupies 4 bytes.

### Why This Structure is Needed:

This architecture provides a clear organization of data on the device, allowing for easy addition, extraction, and deletion of information. Initialization markers help verify data integrity, while segment metadata ensures ease of management and memory optimization. This storage method effectively utilizes available space and minimizes fragmentation, thereby increasing the device's performance.