import random

DATASET_SIZE = 600

paragraph_topics = [
    "Technology continues to influence daily communication and work",
    "Nature contains complex systems that interact in balanced ways",
    "Cities bring together people from different cultures and backgrounds",
    "Learning new skills requires patience and consistent practice",
    "Scientific discoveries often begin with simple observations",
    "Human creativity often appears when people solve practical problems",
    "Digital platforms allow people to collaborate across the world"
]

paragraph_followups = [
    "People study these patterns to better understand how systems evolve",
    "Over time these processes influence society in subtle ways",
    "Researchers continue exploring these ideas from different perspectives",
    "Small discoveries sometimes lead to major innovations",
    "Understanding these systems helps people make better decisions",
    "New questions often appear after the first answers are discovered",
    "Different communities adapt these ideas to their own needs"
]

family_conversations = [
    [
        "Mom asked if everyone was ready for dinner.",
        "Her son replied that he needed a few more minutes to finish his homework.",
        "The father laughed and said that five minutes usually meant fifteen.",
        "Everyone eventually gathered at the table and began talking about their day."
    ],
    [
        "A sister asked her brother if he had borrowed her headphones.",
        "He denied it at first but then remembered they were in his backpack.",
        "She rolled her eyes and told him to return them later.",
        "They both laughed because this situation happened often."
    ],
    [
        "A father asked his daughter how school was today.",
        "She explained that her class worked on a science project.",
        "He listened carefully and asked a few curious questions.",
        "The conversation continued while they prepared dinner together."
    ]
]

friend_conversations = [
    [
        "Two friends met at a small café after work.",
        "One of them asked how the new job was going.",
        "The other explained that it was challenging but exciting.",
        "They spent the next hour sharing stories and making weekend plans."
    ],
    [
        "A group of friends discussed which movie to watch.",
        "Everyone had a different opinion about what sounded interesting.",
        "After some debate they agreed to pick something none of them had seen before.",
        "The decision took longer than the movie itself."
    ],
    [
        "One friend asked another if they wanted to go hiking on Saturday.",
        "The other checked their schedule and said it sounded like a great idea.",
        "They started planning the route and what supplies they should bring.",
        "Soon the plan turned into a small adventure."
    ]
]

dating_conversations = [
    [
        "Two people met for the first time at a quiet restaurant.",
        "They began with simple questions about work and hobbies.",
        "Soon the conversation became relaxed and natural.",
        "By the end of the evening they agreed to meet again."
    ],
    [
        "A man nervously asked his date what kind of music she liked.",
        "She smiled and said she enjoyed many different styles.",
        "They discovered they both liked the same old band.",
        "That small coincidence made the conversation easier."
    ],
    [
        "A woman asked her date what he enjoyed doing on weekends.",
        "He described long walks, cooking experiments, and reading novels.",
        "She laughed and said she enjoyed similar quiet activities.",
        "The shared interests helped them feel more comfortable."
    ]
]

daily_interactions = [
    [
        "A customer asked the cashier if the store had fresh bread.",
        "The cashier pointed toward the bakery section in the back.",
        "The customer thanked her and walked over to the shelves.",
        "The smell of warm bread filled the area."
    ],
    [
        "Someone on the bus asked the passenger next to them if the seat was taken.",
        "The passenger moved their bag and said it was free.",
        "They both nodded politely and looked out the window.",
        "The bus continued moving through the busy streets."
    ],
    [
        "A traveler asked a stranger for directions to the train station.",
        "The stranger explained the route carefully.",
        "The traveler repeated the instructions to make sure they understood.",
        "They thanked each other before walking away."
    ]
]


def generate_paragraph():
    base = random.choice(paragraph_topics)
    sentences = random.randint(3,6)

    text = [base + "."]

    for _ in range(sentences - 1):
        text.append(random.choice(paragraph_followups) + ".")

    return " ".join(text)


def generate_dialogue(pool):
    conversation = random.choice(pool)
    return " ".join(conversation)


def generate_dataset(size):
    paragraphs = []

    for _ in range(size):
        choice = random.choice([
            "paragraph",
            "family",
            "friends",
            "dating",
            "daily"
        ])

        if choice == "paragraph":
            paragraphs.append(generate_paragraph())

        elif choice == "family":
            paragraphs.append(generate_dialogue(family_conversations))

        elif choice == "friends":
            paragraphs.append(generate_dialogue(friend_conversations))

        elif choice == "dating":
            paragraphs.append(generate_dialogue(dating_conversations))

        else:
            paragraphs.append(generate_dialogue(daily_interactions))

    return paragraphs


def save_markdown(data, filename="../data/english_paragraphs.md"):
    with open(filename, "w", encoding="utf-8") as f:
        f.write("# English Paragraphs\n\n")
        for p in data:
            f.write(p + "\n\n")


if __name__ == "__main__":
    dataset = generate_dataset(DATASET_SIZE)
    save_markdown(dataset)
    print(f"Dataset generated with {DATASET_SIZE} paragraphs.")

