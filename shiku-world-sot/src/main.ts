import './style.css';
import { TextWriter } from './text-writer.ts';

const SpeechRecognition =
  window.SpeechRecognition || window.webkitSpeechRecognition;

function start_speech_recognition() {
  const app_container = document.querySelector<HTMLDivElement>('#app');

  if (!app_container) {
    throw new Error('App container not found');
  }
  if (SpeechRecognition) {
    const recognition = new SpeechRecognition();
    const text_writer = new TextWriter(app_container);
    text_writer.full_clear.one(() => {
      recognition.stop();
    });
    recognition.lang = 'en-US';

    recognition.continuous = true;

    recognition.interimResults = true;

    recognition.start();

    recognition.onresult = function (event) {
      let current_result = '';
      console.log(event);
      for (let i = event.resultIndex; i < event.results.length; i++) {
        current_result += event.results[i][0].transcript + ' ';
      }
      current_result = current_result.trim().replace('  ', ' ');
      text_writer.write_text(current_result, event.resultIndex, 'default');
    };

    recognition.onend = function () {
      recognition.onresult = null;
      text_writer.destroy();
      start_speech_recognition();
    };
  } else {
    console.log('Web Speech API is not supported by this browser.');
  }
}

start_speech_recognition();
