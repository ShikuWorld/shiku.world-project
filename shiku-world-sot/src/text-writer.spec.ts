import { TextWriter } from './text-writer';

describe('TextWriter', () => {
  let text_box: HTMLElement;
  let textWriter: TextWriter;

  beforeEach(() => {
    jest.useFakeTimers();
    text_box = document.createElement('div');
    textWriter = new TextWriter(text_box);
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it('should write text to text_box', () => {
    const text = 'Hello, World!';
    textWriter.text_write(text);

    jest.advanceTimersByTime(textWriter.delay * 1000 * text.length);

    const spans = text_box.querySelectorAll('span');
    expect(spans.length).toBe(text.length);

    spans.forEach((span, index) => {
      expect(span.textContent).toBe(text[index]);
    });
  });
});
