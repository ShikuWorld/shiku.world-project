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
    // Get the current result
    const current = event.resultIndex;

    // Get the transcript of the current result
    const transcript = event.results[current][0].transcript;
    console.log(event.results);
    if (app_container) {
      app_container.innerHTML = transcript;
    }
  };
} else {
  console.log('Web Speech API is not supported by this browser.');
}
