import './style.css';

let app_container = document.querySelector<HTMLDivElement>('#app');

const SpeechRecognition =
  window.SpeechRecognition || window.webkitSpeechRecognition;

// Check if the Web Speech API is available
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
    let result = '';

    for (let i = event.resultIndex; i < event.results.length; i++) {
      result += event.results[i][0].transcript + ' ';
    }
    if (app_container) {
      app_container.innerHTML = result;
    }
  };
} else {
  console.log('Web Speech API is not supported by this browser.');
}
