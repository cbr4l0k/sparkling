# Fizzy Bot

Ok, I found [this cool app to self host better looking/working Kanban
app](https://www.fizzy.do/)... this is just a way for me to be able to use it
from my telegram whenever I'm not in my laptop.

I played with the source code to expose the right ports and mount the storage
to inspect the content. Currently I keep working to make in fit my use cases.

I don't want to add any licence to the folder, so just assume it's MIT or
something that let you do whatever you want with it. It's mainly Claude Code
generated; but, once in a while I do the changes if I have the time or if I
really wanna learn how to do something specific.

- [x] View cards, boards, details
- [x] Add comments, close/reopen cards
- [ ] Create cards, move columns, assign/tag
- [ ] Search functionality
- [x] Fix CardStatus showing always "published"

## Changes to the original project

I added a really insecure logger in the `app/controllers/concerns/authentication.rb`/`config/environments/production.rb` so that I'm able to see the magic code to login, without setting up the mailing service or something similar.

The following is the `docker-compose.yml` I wrote to run the migrations and similar stuff to be able to have a reliable service on startup.

```docker-compose
name: fizzy
 
 services:
   fizzy:
     build: ./
     restart: always
     environment:
       - SECRET_KEY_BASE=$SECRET_KEY_BASE
     ports:
       - "3000:3000"
       - "3006:3006"
     volumes:
       - "$PATH_TO_THE_STORAGE_FOLDER/storage:/rails/storage"
     entrypoint: []
     command: >
       bash -c "
         ./bin/rails db:prepare &&
         ./bin/rails db:migrate:cache &&
         ./bin/rails db:migrate:cable &&
         ./bin/rails db:migrate:queue &&
         exec ./bin/thrust ./bin/rails server
       "
```

The `PATH_TO_THE_STORAGE_FOLDER` is just a placeholder I setted up so that I could debug and understand how the database was configured. 

Finally, but maybe most importantly... I turned off the `assume_ssl` and `force_ssl` from the production config because it was easier to deploy to the development alternative, but I'm not configuring ssl certificates for my personal overengineered todo list. 

---

THE TELEGRAM BOT SHOULD WORK ON ITS OWN IF PORT ENV IS CORRECTLY SETTED UP. MOST OF THE ADDITIONAL CHANGES WHERE MADE FOR TESTING/DEBUGGING REASONS.
