document.addEventListener('DOMContentLoaded', function() {
  const buttons = document.querySelectorAll('.mode-selector button');
  const sections = document.querySelectorAll('section');

  buttons.forEach(button => {
    button.addEventListener('click', () => {
      const target = button.getAttribute('data-target');
      sections.forEach(sec => {
        if (sec.id === target) {
          sec.classList.add('active');
        } else {
          sec.classList.remove('active');
        }
      });
    });
  });
});
