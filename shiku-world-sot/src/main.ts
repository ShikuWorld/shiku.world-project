import './style.css';
import { TextWriter } from './text-writer.ts';

const app_container = document.querySelector<HTMLDivElement>('#app');

if (!app_container) {
  throw new Error('App container not found');
}

const SpeechRecognition =
  window.SpeechRecognition || window.webkitSpeechRecognition;

const text_writer = new TextWriter(app_container);

// Check if the Web Speech API is available
function start_speech_recognition() {
  if (SpeechRecognition) {
    // Create a new instance of the webkitSpeechRecognition object
    const recognition = new SpeechRecognition();

    // Set the language of the recognition
    recognition.lang = 'en-US';

    // Set the recognition to continuous, so it keeps listening
    recognition.continuous = true;

    // Set the recognition to return interim results, so we can get results in real-time
    recognition.interimResults = true;

    // Start the recognition
    recognition.start();

    // This event is triggered when the speech recognition service returns a result
    recognition.onresult = function (event) {
      let current_result = '';
      for (let i = event.resultIndex; i < event.results.length; i++) {
        current_result += event.results[i][0].transcript + ' ';
      }
      current_result = current_result.trim().replace('  ', ' ');
      text_writer.write_text(current_result, event.resultIndex, 'default');
    };

    recognition.onend = function () {
      start_speech_recognition();
    };
  } else {
    console.log('Web Speech API is not supported by this browser.');
  }
}

start_speech_recognition();
