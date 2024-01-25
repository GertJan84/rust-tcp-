import asyncio
import random
import time

# TODO: Create new function that sends channels to scout tcp channel/group
# HACK: Use regex instad of string manipulation

usedNames = set() 
channels = {} 

async def get_users(channel):
    reader, writer = await asyncio.open_connection(
        '127.0.0.1', 1234)

    # Get welcome
    await reader.read(1024)
    
    with open('names') as f:
        names = f.readlines()
        names = [name.strip() for name in names]
        availableNames = [name for name in names if name not in usedNames]

        if availableNames:
            name = random.choice(availableNames)
            usedNames.add(name)

            name += '\n'

            # send username
            writer.write(name.encode())
            await writer.drain()

            await reader.read(1024)

            channel += '\n'
            writer.write(channel.encode())
            await writer.drain()

            await reader.read(1024)

            writer.write(b'/users\n')
            await writer.drain()

            data = await reader.read(1024)
            users = [user[2:] for user in data.decode().split('\n')[1:-1]]
            channels[channel[:-1]] = users

            writer.write(b'/exit\n')
            await writer.drain()

            writer.write(b'\n')
            await writer.drain()


async def ask_channels():
    reader, writer = await asyncio.open_connection(
        '127.0.0.1', 1234)

    # Get welcome
    data = await reader.read(1024)
    message = 'bot\n'

    # send username
    writer.write(message.encode())
    await writer.drain()

    await reader.read(1024)

    while True:
        writer.write(b'/groups\n')
        await writer.drain()

        data = await reader.read(1024)
        res_channels = [channel.split(" ")[1] for channel in data.decode().split("\n")[1:-1]] # get groups

        for channel in res_channels:
            if channel == 'None':
                continue

            channel = channel.split(" |")[0]

            channels[channel] = []
            await get_users(channel)

        time.sleep(3)

        print(channels)
        
        # TODO: Fix this
        # writer.write(b'scout\n')
        # await writer.drain()

        # await reader.read(1024)

        # for group, users in channels.items():

            # channel_announce = f'**** {group} ****\n'
            
            # writer.write(channel_announce.encode())
            # await writer.drain()

            # for user in users:
                # user += '\n'
                # writer.write(user.encode())
                # await writer.drain()


        # writer.write(b'/disconnect\n')
        # await writer.drain()

asyncio.run(ask_channels())
