use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;
use crate::Route;

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        // Get current user for message bubble coloring
        let current_username = ctx
            .link()
            .context::<User>(Callback::noop())
            .map(|(u, _)| u.username.borrow().clone())
            .unwrap_or_default();

        html! {
            <div class="flex w-screen">
                <div class="flex-none w-56 h-screen bg-gradient-to-b from-gray-100 to-blue-100">
                    <div class="text-xl p-3 flex flex-col gap-2">
                        {"Users"}
                        <span class="text-xs text-blue-600">{"🌟 Active Now"}</span>
                    </div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-white rounded-lg p-2 shadow-sm items-center">
                                    <div>
                                        <img class="w-12 h-12 rounded-full border-2 border-blue-200" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div class="font-semibold">{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-400 italic">
                                            {"Hi there! 👋"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col" style="background: repeating-linear-gradient(135deg,#f0f4ff 0 20px,#e9e9f9 20px 40px);">
                    <div class="w-full h-14 border-b-2 border-gray-300 flex items-center justify-between">
                        <div class="text-xl p-3 flex items-center gap-2">
                            {"💬 Chat!"}
                            <span class="ml-2 text-sm text-blue-400 animate-bounce">{"Welcome to the fun zone! 🎉"}</span>
                        </div>
                        <div class="mr-4 text-xs text-gray-500 italic">{"Tip: Try sending an emoji or a .gif URL!"}</div>
                    </div>
                    <div class="w-full grow overflow-auto border-b-2 border-gray-300 px-2 py-4">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from);
                                let avatar = user.map(|u| u.avatar.clone()).unwrap_or_else(|| "https://cdn-icons-png.flaticon.com/512/4712/4712035.png".to_string());
                                let is_me = m.from == current_username;
                                let bubble_class = if is_me {
                                    "bg-blue-200 text-right ml-auto"
                                } else {
                                    "bg-gray-100"
                                };
                                let text_class = if is_me {
                                    "text-blue-900"
                                } else {
                                    "text-gray-700"
                                };
                                html!{
                                    <div class={format!("flex items-end w-3/6 m-4 rounded-tl-lg rounded-tr-lg rounded-br-lg shadow-sm {}", if is_me { "flex-row-reverse" } else { "" })}>
                                        <img class="w-8 h-8 rounded-full m-3 border border-blue-100" src={avatar} alt="avatar"/>
                                        <div class={format!("p-3 rounded-lg {}", bubble_class)}>
                                            <div class={format!("text-sm font-bold {}", text_class)}>
                                                {if is_me { "You".to_string() } else { m.from.clone() }}
                                            </div>
                                            <div class={format!("text-xs mt-1 {}", text_class)}>
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html!{ <img class="mt-3 rounded" src={m.message.clone()}/> }
                                                    } else {
                                                        html!{ <span>{m.message.clone()}</span> }
                                                    }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-18 flex flex-col px-3 py-2 items-center bg-white bg-opacity-80">
                        <div class="w-full flex items-center">
                            <input ref={self.chat_input.clone()} type="text" placeholder="Type your message and hit Enter 🚀" class="block w-full py-2 pl-4 mx-3 bg-gray-100 rounded-full outline-none focus:text-gray-700" name="message" required=true />
                            <button onclick={submit} class="p-3 shadow-sm bg-blue-600 w-10 h-10 rounded-full flex justify-center items-center color-white hover:bg-blue-700 transition">
                                <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                    <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                                </svg>
                            </button>
                        </div>
                        <div class="w-full text-xs text-gray-400 mt-1 text-center italic">
                            {"💡 Tip of the Day: Spread kindness, one message at a time!"}
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
