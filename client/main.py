import asyncio
import random

usedNames = []
channels = {} #hannel_name = [users]

async def connect_to_channel(channel):
    reader, writer = await asyncio.open_connection(
        '127.0.0.1', 1234)

    # Get welcome
    await reader.read(1024)

    with open('names') as f:
        names = f.readlines()
        name = ''
        while name and name not in usedNames:
            name = random.choice(names)
        name += '\n'

    # send username
    print(f'Connected as: {name!r}')
    writer.write(name.encode())
    await writer.drain()

    await reader.read(1024)

    print(f'Join {channel!r}')
    channel += '\n'
    writer.write(channel.encode())
    await writer.drain()


async def ask_channels():
    reader, writer = await asyncio.open_connection(
        '127.0.0.1', 1234)

    # Get welcome
    data = await reader.read(1024)
    print(f'{data.decode()}')

    message = 'bot\n'

    # send username
    print(f'Send: {message!r}')
    writer.write(message.encode())
    await writer.drain()

    # Get channels
    data = await reader.read(1024)
    print(f'Received: {data.decode()}')

    # Send channel room
    res_channels = [channel[2:] for channel in data.decode().split("\n")[3:-1]]
    for channel in res_channels:

        print(channel)

        if channel == 'None':
            continue

        channel = channel.split(" |")[0]

        if channel in channels:
            continue

        channels[channel] = []
    print(channels)


asyncio.run(ask_channels())
