<script>
    import { onMount } from "svelte";
    let messages = [{ text: "Hi, ask me anything!", user: false }];
    let input = "";
    let error = "";

    async function getGptResponse(chatData) {
        try {
            let response = await fetch("/api/chat", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(chatData),
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            let data = await response.json();
            console.log(data);
            return data;
        } catch (err) {
            error = err.message;
            console.error("Error occurred during fetch:", err);
        }
    }

    async function addMessage() {
        if (!input) return;
        messages = [...messages, { text: input, user: true }];
        let response = await getGptResponse({ messages });
        input = "";
        if (response) {
            messages = [...messages, { text: response, user: false }];
        } else {
            console.error(
                "Error occurred while getting GPT response. Error: ",
                error
            );
        }
    }
</script>

<h2>Chat with ChatGPT!</h2>
<div id="chat-window">
    {#each messages as message (message)}
        <div class={message.user ? "user-message" : "gpt-message"}>
            {message.text}
        </div>
    {/each}
</div>
<form on:submit|preventDefault={addMessage} class="prompt">
    <input bind:value={input} placeholder="Type a message..." />
    <button type="submit">Send</button>
</form>

<style>
    body {
        font-family: "Arial", sans-serif;
        background-color: #1a1a1a;
        color: #f1f1f1; /* Brightened text color for better visibility */
    }
    #chat-window {
        max-width: 90%;
        height: 90%;
        overflow: auto;
        border: 1px solid #333333;
        padding: 15px;
        margin: 20px auto;
        background-color: rgba(34, 34, 34, 0.9);
        border-radius: 8px;
        box-shadow: 0 2px 5px rgba(0, 0, 0, 0.3);
    }
    .prompt {
        max-width: 90%;
        display: flex;
        margin: 10px auto;
        border-radius: 8px;
        overflow: hidden; /* This will help contain the child elements */
    }
    input {
        flex-grow: 1; /* Makes the input field take up available space */
        border: none;
        padding: 10px;
        border-right: 1px solid #333333; /* Added a right border for distinction */
        border-radius: 8px 0 0 8px;
        background-color: #333333;
        color: #f1f1f1;
        box-sizing: border-box; /* Makes sure padding and border are included in total width/height */
    }
    button {
        border: none;
        background-color: #007bff;
        color: white;
        padding: 10px 16px; /* Adjusted padding for better alignment */
        border-radius: 0 8px 8px 0;
        cursor: pointer;
        box-sizing: border-box; /* Makes sure padding and border are included in total width/height */
    }
    .user-message,
    .gpt-message {
        margin: 10px;
        padding: 10px;
        border-radius: 8px;
        color: #f1f1f1;
    }
    .user-message {
        background-color: #3a3a3a;
        align-self: flex-end;
        border: 1px solid #444444;
        color: white;
    }
    .gpt-message {
        background-color: #2a2a2a;
        border: 1px solid #444444;
    }
    h2 {
        text-align: center;
        margin-top: 20px;
    }
</style>
