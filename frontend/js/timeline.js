class Timeline {
    constructor() {
        this.dynasties = [];
        this.currentIndex = 0;
        this.isPlaying = false;
        this.playInterval = null;
        this.onDynastyChange = null;
    }

    init(dynasties) {
        this.dynasties = dynasties.sort((a, b) => a.start_year - b.start_year);
        
        const slider = document.getElementById('timelineSlider');
        slider.max = this.dynasties.length;
        slider.min = 1;
        slider.value = 1;
        
        slider.addEventListener('input', (e) => {
            this.setIndex(parseInt(e.target.value) - 1);
        });

        document.getElementById('playBtn').addEventListener('click', () => this.togglePlay());
        document.getElementById('prevBtn').addEventListener('click', () => this.prev());
        document.getElementById('nextBtn').addEventListener('click', () => this.next());

        this.renderLabels();
        this.updateDisplay();
    }

    renderLabels() {
        const labels = document.getElementById('timelineLabels');
        labels.innerHTML = '';
        
        const step = Math.ceil(this.dynasties.length / 7);
        
        for (let i = 0; i < this.dynasties.length; i += step) {
            const label = document.createElement('span');
            label.textContent = this.dynasties[i].name;
            labels.appendChild(label);
        }
    }

    setIndex(index) {
        this.currentIndex = Math.max(0, Math.min(this.dynasties.length - 1, index));
        document.getElementById('timelineSlider').value = this.currentIndex + 1;
        this.updateDisplay();
        
        if (this.onDynastyChange) {
            this.onDynastyChange(this.dynasties[this.currentIndex]);
        }
    }

    updateDisplay() {
        const dynasty = this.dynasties[this.currentIndex];
        if (dynasty) {
            document.getElementById('currentDynasty').textContent = 
                `${dynasty.name} (${dynasty.start_year > 0 ? '公元' : '公元前'}${Math.abs(dynasty.start_year)}年)`;
        }
    }

    togglePlay() {
        if (this.isPlaying) {
            this.stop();
        } else {
            this.play();
        }
    }

    play() {
        this.isPlaying = true;
        document.getElementById('playBtn').textContent = '⏸';
        
        this.playInterval = setInterval(() => {
            if (this.currentIndex < this.dynasties.length - 1) {
                this.next();
            } else {
                this.stop();
            }
        }, 2000);
    }

    stop() {
        this.isPlaying = false;
        document.getElementById('playBtn').textContent = '▶';
        
        if (this.playInterval) {
            clearInterval(this.playInterval);
            this.playInterval = null;
        }
    }

    next() {
        if (this.currentIndex < this.dynasties.length - 1) {
            this.setIndex(this.currentIndex + 1);
        }
    }

    prev() {
        if (this.currentIndex > 0) {
            this.setIndex(this.currentIndex - 1);
        }
    }

    setOnDynastyChange(callback) {
        this.onDynastyChange = callback;
    }

    getCurrentDynasty() {
        return this.dynasties[this.currentIndex];
    }
}
