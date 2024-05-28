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

  it('get pages from text function should work as expected', () => {
    const text = 'Hello, World!                                     ';
    expect(textWriter.get_pages_from_text(text)).toEqual([text]);

    const text2 =
      'Lorem ipsum dolor sit amet, consetetur sadipscingelitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.';
    const expected_text2 =
      'Lorem ipsum dolor sit amet, consetetur            sadipscingelitr, sed diam nonumy eirmod tempor    invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.                                ';
    expect(textWriter.get_pages_from_text(text2)).toEqual([expected_text2]);

    const text3 =
      'Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. test foo yo. Hm....';
    const pages = textWriter.get_pages_from_text(text3);
    expect(pages[0].length).toBe(300);
    expect(pages[1].length).toBe(300);
    expect(pages).toEqual([
      'Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut  labore et dolore magna aliquyam erat, sed diam    voluptua. At vero eos et accusam et justo duo     dolores et ea rebum. Stet clita kasd gubergren, nosea takimata sanctus est Lorem ipsum dolor sit    ',
      'amet. Lorem ipsum dolor sit amet, consetetur      sadipscing elitr, sed diam nonumy eirmod tempor   invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justoduo dolores et ea rebum. Stet clita kasd          gubergren, no sea takimata sanctus est Lorem ipsum',
      'dolor sit amet. test foo yo. Hm....               ',
    ]);
  });

  it('should write the amount of text to the text box in the correct amount of time', () => {
    const text = 'Hello, World!';
    textWriter.write_text(text, 0);

    jest.advanceTimersByTime(textWriter.add_char_delay_in_ms * 6);

    const spans = text_box.querySelectorAll('span');
    expect(spans.length).toBe(7);

    jest.advanceTimersByTime(textWriter.add_char_delay_in_ms * text.length);
    expect(text_box.querySelectorAll('span').length).toBe(13);
    spans.forEach((span, index) => {
      expect(span.textContent).toBe(text[index]);
    });
  });

  it('should create new line if current text is too much and break sensibly by word', () => {
    const text =
      'Lorem ipsum dolor sit amet, consetetur sadipscingelitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.';
    textWriter.write_text(text, 0);

    jest.advanceTimersByTime(textWriter.add_char_delay_in_ms * text.length);
    const lines = text_box.querySelectorAll('p');
    expect(lines.length).toBe(4);
    const line_should = 'Lorem ipsum dolor sit amet, consetetur            ';
    lines[0].querySelectorAll('span').forEach((span, index) => {
      expect(span.textContent).toBe(line_should[index]);
    });
  });

  it('should create new line after line delay time elapsed', () => {
    const text = 'Hello, World!';
    textWriter.write_text(text, 0);

    jest.advanceTimersByTime(textWriter.add_char_delay_in_ms * text.length);
    expect(text_box.querySelectorAll('p').length).toBe(1);
    expect(
      text_box.querySelectorAll('p')[0].querySelectorAll('span').length,
    ).toBe(13);
    jest.advanceTimersByTime(textWriter.line_delay_in_ms * 1.1);

    expect(textWriter.current_text).toBe('Hello, World!\n');
    expect(textWriter.pages[0]).toBe(
      'Hello, World!                                     ',
    );
    expect(text_box.querySelectorAll('p')[0].textContent).toBe('Hello, World!');
    textWriter.write_text(text, 1);
    expect(textWriter.current_text).toBe('Hello, World!\nHello, World!');
    expect(textWriter.pages[0]).toBe(
      'Hello, World!                                     Hello, World!                                     ',
    );
    jest.advanceTimersByTime(
      textWriter.add_char_delay_in_ms * (text.length + 1),
    );
    expect(text_box.querySelectorAll('p').length).toBe(2);
    expect(text_box.querySelectorAll('p')[0].textContent).toBe(
      'Hello, World!                                     ',
    );
    expect(text_box.querySelectorAll('p')[1].textContent).toBe('Hello, World!');
    const lines = text_box.querySelectorAll('p');
    expect(lines.length).toBe(2);
    expect(lines[0].querySelectorAll('span').length).toBe(
      textWriter.max_chars_per_line,
    );
    expect(lines[1].querySelectorAll('span').length).toBe(13);
  });

  it('when writing text before the line_delay_in_ms is up, the current text being written should be replaced', () => {
    const text = 'Hello, World!';
    const new_text = 'Hemlo, Kevin, nice day?';
    textWriter.write_text(text, 0);

    jest.advanceTimersByTime(textWriter.add_char_delay_in_ms * 6);

    const spans = text_box.querySelectorAll('span');
    expect(spans.length).toBe(7);

    textWriter.write_text(new_text, 0);
    text_box.querySelectorAll('span').forEach((span, index) => {
      expect(span.textContent).toBe(new_text[index]);
    });

    jest.advanceTimersByTime(textWriter.add_char_delay_in_ms * new_text.length);
    const spans_after = text_box.querySelectorAll('span');
    expect(spans_after.length).toBe(23);
    spans_after.forEach((span, index) => {
      expect(span.textContent).toBe(new_text[index] ? new_text[index] : ' ');
    });
  });

  it('should wait go_to_next_page_time_in_ms and then remove all spans once too much text exists', () => {
    const text = `Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum.`;
    textWriter.write_text(text, 0);
    const pages = textWriter.get_pages_from_text(text);

    jest.advanceTimersByTime(textWriter.add_char_delay_in_ms * pages[0].length);
    expect(text_box.querySelectorAll('p').length).toBe(6);
    text_box.querySelectorAll('span').forEach((span, index) => {
      expect(span.textContent).toBe(pages[0][index]);
    });

    jest.advanceTimersByTime(textWriter.go_to_next_page_time_in_ms * 1.1);

    expect(text_box.querySelectorAll('p').length).toBe(1);
    jest.advanceTimersByTime(textWriter.add_char_delay_in_ms * pages[1].length);
    expect(text_box.querySelectorAll('p').length).toBe(5);
    text_box.querySelectorAll('span').forEach((span, index) => {
      expect(span.textContent).toBe(pages[1][index]);
    });
  });

  it('should pass all expectations found in bug_1', () => {
    const sentences = [
      'another egg will be eaten at dawn and it will be very interesting',
      'i bought some water yesterday and it was delicious',
      'this seems to be working',
    ];
    const non_waiting_sentences = [
      'wow that is out of the way lets see what we will be doing next as there is no way that this text could grow any longer',
      'test is good',
      'test is fine',
      'everything is awesome when I rhyme',
      'Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.',
    ];

    sentences.forEach((sentence, index) => {
      const words = sentence.split(' ');
      let input = '';
      words.forEach((word) => {
        input += word + ' ';
        textWriter.write_text(input, index);
        jest.advanceTimersByTime(
          textWriter.add_char_delay_in_ms * (words.length + 1),
        );
      });
      jest.advanceTimersByTime(textWriter.line_delay_in_ms * 1.1);
    });
    const l = text_box.querySelectorAll('p');
    expect(textWriter.pages[0]).toBe(
      'Another egg will be eaten at dawn and it will be  very interesting.                                 I bought some water yesterday and it was          delicious.                                        This seems to be working.                         ',
    );

    expect(l.length).toBe(5);
    expect(l[0].textContent).toBe(
      'Another egg will be eaten at dawn and it will be  ',
    );
    expect(l[1].textContent).toBe(
      'very interesting.                                 ',
    );
    expect(l[2].textContent).toBe(
      'I bought some water yesterday and it was          ',
    );
    expect(l[3].textContent).toBe(
      'delicious.                                        ',
    );
    expect(l[4].textContent).toBe('This seems to be working');

    non_waiting_sentences.forEach((sentence, index) => {
      const words = sentence.split(' ');
      let input = '';
      words.forEach((word) => {
        input += word + ' ';
        textWriter.write_text(input, index);
        jest.advanceTimersByTime(
          textWriter.add_char_delay_in_ms * (words.length + 1),
        );
      });
    });
    expect(textWriter.pages[1]).toBe(
      'be doing next as there is no way that this text   could grow any longer. Test is good. Test is fine.Everything is awesome when I rhyme. Lorem ipsum   dolor sit amet, consetetur sadipscing elitr, sed  diam nonumy eirmod tempor invidunt ut labore et   dolore magna aliquyam erat, sed diam voluptua. At ',
    );
    expect(textWriter.pages[2]).toBe(
      'vero eos et accusam et justo duo dolores et ea    rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem     ipsum dolor sit amet, consetetur sadipscing elitr,sed diam nonumy eirmod tempor invidunt ut labore  et dolore magna aliquyam erat, sed diam voluptua. ',
    );
    expect(textWriter.pages[3]).toBe(
      'At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.           ',
    );
  });
});
