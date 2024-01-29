import asyncio
import random
import re

usedNames = set() 
channels = {} 

getUsername = re.compile(r"(?<=\*\*).*(?= joined| left)")
groupReg = re.compile(r"(?<=\- ).*(?= \|)")
userReg = re.compile(r"(?<=\- ).*(?=\n)")


async def get_users(channel):
    reader, writer = await asyncio.open_connection(
        '127.0.0.1', 1234)

    # Get welcome
    await reader.read(1024)
    
    with open('names') as f:
        names = f.readlines()
        names = [name.strip() for name in names]
        availableNames = [name + " (bot)" for name in names if name + " (bot)" not in usedNames]

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

            
            while True:
                past_messages = await reader.read(1024)

                userName = getUsername.search(past_messages.decode())
                name = name[:-1]

                if not userName:
                    continue

                userName = userName.group()

                if userName == name:

                    writer.write(b'/users\n')
                    await writer.drain()

                    data = await reader.read(1024)
                    users = userReg.findall(data.decode())
                    if name in users:
                       users.remove(name)
                    channels[channel[:-1]] = users
                else:
                    if userName in channels[channel[:-1]]:
                        print(f"{userName} left")
                        channels[channel[:-1]].remove(userName)
                    else:
                        print(f"{userName} joined")
                        channels[channel[:-1]].append(userName)

                if len(channels[channel[:-1]]) == 0:
                    writer.write(b'/exit\n')
                    await writer.drain()
                    

                await asyncio.sleep(3)



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


    async with asyncio.TaskGroup() as tg:

        while True:
            writer.write(b'/groups\n')
            await writer.drain()

            data = await reader.read(1024)
            res_channels = groupReg.findall(data.decode())

            for channel in res_channels:
                if channel == 'None':
                    continue

                channel = channel.split(" |")[0]

                if channel == "scout":
                    continue


                if channel in set(channels.keys()):
                    if not len(channels[channel]):
                        del(channels[channel])
                    continue

                channels[channel] = []

                tg.create_task(get_users(channel))

            # Print to scout
            print(channels)
            
            writer.write(b'scout\n')
            await writer.drain()

            await reader.read(1024)

            for group, users in channels.items():

                channel_announce = f'**** {group} ****\n'
                
                writer.write(channel_announce.encode())
                await writer.drain()

                for user in users:
                    user += '\n'
                    writer.write(user.encode())
                    await writer.drain()

            writer.write(b'/leave\n')
            await writer.drain()

            await asyncio.sleep(3)

asyncio.run(ask_channels())
