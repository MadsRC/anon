use crate::detection::{EntityDetector, EntityType, ner::GlinerDetector};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PerformanceBenchmark {
    pub model_name: String,
    pub test_name: String,
    pub total_chars_processed: usize,
    pub total_processing_time: Duration,
    pub chars_per_second: f64,
    pub avg_latency_per_text: Duration,
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub texts_processed: usize,
    pub total_entities_detected: usize,
    pub entities_per_second: f64,
}

pub struct PerformanceTester {
    test_texts: Vec<String>,
    models: Vec<(String, String, String)>, // (name, tokenizer_path, model_path)
}

impl PerformanceTester {
    pub fn new() -> Self {
        Self {
            test_texts: Vec::new(),
            models: Vec::new(),
        }
    }

    pub fn add_model(&mut self, name: String, tokenizer_path: String, model_path: String) {
        self.models.push((name, tokenizer_path, model_path));
    }

    pub fn with_synthetic_texts(&mut self, count: usize) -> &mut Self {
        self.test_texts.extend(self.generate_synthetic_texts(count));
        self
    }

    pub fn with_real_world_texts(&mut self) -> &mut Self {
        self.test_texts.extend(self.get_real_world_texts());
        self
    }

    pub fn with_stress_test_texts(&mut self) -> &mut Self {
        self.test_texts.extend(self.generate_stress_test_texts());
        self
    }

    fn generate_synthetic_texts(&self, count: usize) -> Vec<String> {
        let templates = vec![
            "Contact {person} at {org} via {email} or call {phone}.",
            "{person} from {org} will be presenting in {location} next week.",
            "The meeting with {person} and {person2} from {org} is scheduled for {location}.",
            "Please send the documents to {email} and {email2} by tomorrow.",
            "Dr. {person} at {org} can be reached at {phone} or {email}.",
            "{person} works at {org} in {location} and specializes in AI research.",
            "The conference in {location} will feature speakers from {org} and {org2}.",
            "For technical support, contact {person} at {email} or call {phone}.",
        ];

        let people = vec![
            "John Smith",
            "Sarah Johnson",
            "Michael Chen",
            "Emily Davis",
            "David Wilson",
            "Lisa Brown",
            "James Miller",
            "Anna Garcia",
        ];
        let orgs = vec![
            "Microsoft",
            "Google",
            "Apple",
            "Amazon",
            "Tesla",
            "Meta",
            "OpenAI",
            "Anthropic",
            "Stanford University",
            "MIT",
        ];
        let locations = vec![
            "New York",
            "San Francisco",
            "London",
            "Tokyo",
            "Berlin",
            "Toronto",
            "Sydney",
            "Singapore",
        ];
        let emails = vec![
            "john.smith@company.com",
            "s.johnson@org.edu",
            "m.chen@tech.co",
            "emily@startup.io",
        ];
        let phones = vec![
            "(555) 123-4567",
            "(650) 555-0123",
            "+1-800-555-9999",
            "(415) 555-7890",
        ];

        let mut texts = Vec::new();
        for i in 0..count {
            let template = &templates[i % templates.len()];
            let text = template
                .replace("{person}", &people[i % people.len()])
                .replace("{person2}", &people[(i + 1) % people.len()])
                .replace("{org}", &orgs[i % orgs.len()])
                .replace("{org2}", &orgs[(i + 1) % orgs.len()])
                .replace("{location}", &locations[i % locations.len()])
                .replace("{email}", &emails[i % emails.len()])
                .replace("{email2}", &emails[(i + 1) % emails.len()])
                .replace("{phone}", &phones[i % phones.len()]);

            texts.push(text);
        }
        texts
    }

    fn get_real_world_texts(&self) -> Vec<String> {
        vec![
            "Dr. Sarah Chen from Stanford University will present her groundbreaking research on artificial intelligence at the upcoming International Conference on Machine Learning in Vancouver. Attendees can register by contacting the conference organizers at icml2024@conference.org or calling (604) 555-0198. Dr. Chen's work focuses on natural language processing and has been published in top-tier journals. For media inquiries, please reach out to s.chen@stanford.edu directly.".to_string(),

            "The quarterly board meeting will be held at Microsoft's headquarters in Redmond, Washington, on March 15th. Key attendees include CEO Satya Nadella, CFO Amy Hood, and CTO Kevin Scott. The agenda covers Azure growth projections, OpenAI partnership updates, and sustainability initiatives. Remote participants can join via Teams using the link provided by executive.assistant@microsoft.com. Questions should be directed to the board secretary at (425) 555-0147.".to_string(),

            "Tesla's Gigafactory Texas has reached a new production milestone, manufacturing over 5,000 Model Y vehicles per week. Factory Director James Rodriguez announced the achievement during a press conference in Austin. The facility, which employs over 20,000 workers, has been instrumental in Tesla's expansion plans. Investors can schedule facility tours by emailing tours@tesla.com or calling the Austin office at (512) 555-0299. The success has been attributed to innovative manufacturing processes and strong local partnerships.".to_string(),

            "Breaking: A major data breach at Equifax has potentially exposed personal information of 147 million consumers. The company's security team, led by Chief Information Security Officer Susan Mauldin, discovered the breach during routine security monitoring. Affected customers should contact Equifax immediately at breach.support@equifax.com or call the dedicated helpline at 1-800-525-6285. The Federal Trade Commission and multiple state attorneys general are investigating the incident. Equifax has set up a website at www.equifaxsecurity2017.com for affected consumers to check if their information was compromised.".to_string(),

            "Amazon Web Services (AWS) announced a new data center region opening in Mumbai, India, expanding their global cloud infrastructure. The region, led by Regional Director Priya Sharma, will provide low-latency cloud services to customers across South Asia. Enterprise customers interested in migrating to the new region can contact the AWS sales team at aws.india@amazon.com or schedule a consultation by calling +91-80-6749-4000. The Mumbai region joins AWS's existing presence in Hyderabad and supports the company's commitment to the Indian market.".to_string(),

            "Apple's annual Worldwide Developers Conference (WWDC) will take place from June 5-9 at the McEnery Convention Center in San Jose, California. Software engineering chief Craig Federighi will keynote the opening session, unveiling updates to iOS, macOS, watchOS, and tvOS. Developers can register at developer.apple.com or contact Apple Developer Relations at devrelations@apple.com for partnership opportunities. Media credentials are available through press@apple.com, and the event will be livestreamed on Apple's website and Apple TV.".to_string(),
        ]
    }

    fn generate_stress_test_texts(&self) -> Vec<String> {
        vec![
            // Long text with many entities
            format!("The annual technology summit brought together industry leaders from across Silicon Valley and beyond. Attendees included Tim Cook from Apple (tcook@apple.com, 408-996-1010), Sundar Pichai from Google (s.pichai@google.com, 650-253-0000), Satya Nadella from Microsoft (satya@microsoft.com, 425-882-8080), and Jeff Bezos from Amazon (jeff@amazon.com, 206-266-1000). The event took place in San Francisco, California, with satellite locations in New York City, London, England, Tokyo, Japan, and Berlin, Germany. Key topics included artificial intelligence, quantum computing, sustainable technology, and digital transformation. Speakers also included Dr. Sarah Johnson from MIT (s.johnson@mit.edu), Professor Michael Chen from Stanford University (m.chen@stanford.edu), Dr. Emily Davis from Carnegie Mellon (e.davis@cmu.edu), and Dr. David Wilson from UC Berkeley (d.wilson@berkeley.edu). Networking sessions were held at various venues including the Moscone Center, Marriott Marquis, and the Palace Hotel. Contact information for follow-up meetings can be obtained by emailing summit.organizers@techsummit.org or calling the main office at (415) 555-TECH."),

            // Text with challenging entity boundaries
            "Dr. John Smith-Johnson, M.D., Ph.D. from UCLA Medical Center-West Los Angeles can be reached at j.smith-johnson@ucla-medical.edu or (310) 825-9111 ext. 5547. His research on COVID-19 variants has been published in Nature Medicine and The New England Journal of Medicine.".to_string(),

            // Text with mixed languages and special characters
            "María José González from Universidad Autónoma de México collaborates with François Müller from ETH Zürich on quantum computing research. Contact: m.gonzalez@unam.mx and f.mueller@ethz.ch. Phone: +52-55-5622-2511 and +41-44-632-1111.".to_string(),

            // Dense entity text
            "Meeting: Alice (alice@a.com, 555-0001), Bob (bob@b.com, 555-0002), Charlie (charlie@c.com, 555-0003), Diana (diana@d.com, 555-0004), Eve (eve@e.com, 555-0005) from Apple, Google, Microsoft, Amazon, Tesla respectively in NYC, SF, Seattle, Austin, Berlin on Monday.".to_string(),

            // Very short texts
            "Call John at 555-1234.".to_string(),
            "Email: jane@company.com".to_string(),
            "Dr. Smith from MIT".to_string(),
        ]
    }

    pub fn run_comprehensive_benchmarks(&self) -> Vec<PerformanceBenchmark> {
        let mut results = Vec::new();

        for (model_name, tokenizer_path, model_path) in &self.models {
            println!("🚀 Benchmarking model: {}", model_name);

            let detector_result = GlinerDetector::new(
                tokenizer_path,
                model_path,
                vec![
                    EntityType::Person,
                    EntityType::Organization,
                    EntityType::Location,
                    EntityType::Email,
                    EntityType::PhoneNumber,
                ],
            );

            match detector_result {
                Ok(mut detector) => {
                    // Test different scenarios
                    results.extend(self.run_throughput_test(model_name, &mut detector));
                    results.extend(self.run_latency_test(model_name, &mut detector));
                    results.extend(self.run_scale_test(model_name, &mut detector));
                }
                Err(e) => {
                    println!("❌ Failed to load model {}: {}", model_name, e);
                }
            }
        }

        results
    }

    fn run_throughput_test(
        &self,
        model_name: &str,
        detector: &mut GlinerDetector,
    ) -> Vec<PerformanceBenchmark> {
        println!("  📊 Running throughput test...");

        let start_time = Instant::now();
        let mut total_chars = 0;
        let mut total_entities = 0;
        let mut processing_times = Vec::new();

        for text in &self.test_texts {
            let text_start = Instant::now();
            match detector.detect(text) {
                Ok(entities) => {
                    let text_time = text_start.elapsed();
                    total_chars += text.len();
                    total_entities += entities.len();
                    processing_times.push(text_time);
                }
                Err(_) => {
                    processing_times.push(text_start.elapsed());
                }
            }
        }

        let total_time = start_time.elapsed();
        let chars_per_second = total_chars as f64 / total_time.as_secs_f64();
        let entities_per_second = total_entities as f64 / total_time.as_secs_f64();

        let avg_latency = Duration::from_nanos(
            processing_times
                .iter()
                .map(|d| d.as_nanos() as u64)
                .sum::<u64>()
                / processing_times.len() as u64,
        );
        let min_latency = processing_times.iter().min().cloned().unwrap_or_default();
        let max_latency = processing_times.iter().max().cloned().unwrap_or_default();

        vec![PerformanceBenchmark {
            model_name: model_name.to_string(),
            test_name: "Throughput Test".to_string(),
            total_chars_processed: total_chars,
            total_processing_time: total_time,
            chars_per_second,
            avg_latency_per_text: avg_latency,
            min_latency,
            max_latency,
            texts_processed: self.test_texts.len(),
            total_entities_detected: total_entities,
            entities_per_second,
        }]
    }

    fn run_latency_test(
        &self,
        model_name: &str,
        detector: &mut GlinerDetector,
    ) -> Vec<PerformanceBenchmark> {
        println!("  ⚡ Running latency test...");

        // Test with different text sizes
        let size_categories = vec![
            (
                "Small (< 100 chars)",
                self.test_texts
                    .iter()
                    .filter(|t| t.len() < 100)
                    .collect::<Vec<_>>(),
            ),
            (
                "Medium (100-500 chars)",
                self.test_texts
                    .iter()
                    .filter(|t| t.len() >= 100 && t.len() < 500)
                    .collect::<Vec<_>>(),
            ),
            (
                "Large (> 500 chars)",
                self.test_texts
                    .iter()
                    .filter(|t| t.len() >= 500)
                    .collect::<Vec<_>>(),
            ),
        ];

        let mut results = Vec::new();

        for (category_name, texts) in size_categories {
            if texts.is_empty() {
                continue;
            }

            let mut processing_times = Vec::new();
            let mut total_chars = 0;
            let mut total_entities = 0;

            for text in &texts {
                let start = Instant::now();
                match detector.detect(text) {
                    Ok(entities) => {
                        let duration = start.elapsed();
                        processing_times.push(duration);
                        total_chars += text.len();
                        total_entities += entities.len();
                    }
                    Err(_) => {
                        processing_times.push(start.elapsed());
                    }
                }
            }

            let total_time = processing_times.iter().sum::<Duration>();
            let avg_latency = Duration::from_nanos(
                processing_times
                    .iter()
                    .map(|d| d.as_nanos() as u64)
                    .sum::<u64>()
                    / processing_times.len() as u64,
            );
            let min_latency = processing_times.iter().min().cloned().unwrap_or_default();
            let max_latency = processing_times.iter().max().cloned().unwrap_or_default();

            results.push(PerformanceBenchmark {
                model_name: model_name.to_string(),
                test_name: format!("Latency Test - {}", category_name),
                total_chars_processed: total_chars,
                total_processing_time: total_time,
                chars_per_second: total_chars as f64 / total_time.as_secs_f64(),
                avg_latency_per_text: avg_latency,
                min_latency,
                max_latency,
                texts_processed: texts.len(),
                total_entities_detected: total_entities,
                entities_per_second: total_entities as f64 / total_time.as_secs_f64(),
            });
        }

        results
    }

    fn run_scale_test(
        &self,
        model_name: &str,
        detector: &mut GlinerDetector,
    ) -> Vec<PerformanceBenchmark> {
        println!("  📈 Running scale test...");

        // Test with increasing batch sizes
        let batch_sizes = vec![1, 10, 50, 100];
        let mut results = Vec::new();

        for &batch_size in &batch_sizes {
            if batch_size > self.test_texts.len() {
                continue;
            }

            let test_batch = &self.test_texts[..batch_size];
            let start_time = Instant::now();
            let mut total_chars = 0;
            let mut total_entities = 0;

            for text in test_batch {
                match detector.detect(text) {
                    Ok(entities) => {
                        total_chars += text.len();
                        total_entities += entities.len();
                    }
                    Err(_) => {}
                }
            }

            let total_time = start_time.elapsed();

            results.push(PerformanceBenchmark {
                model_name: model_name.to_string(),
                test_name: format!("Scale Test - {} texts", batch_size),
                total_chars_processed: total_chars,
                total_processing_time: total_time,
                chars_per_second: total_chars as f64 / total_time.as_secs_f64(),
                avg_latency_per_text: Duration::from_nanos(
                    total_time.as_nanos() as u64 / batch_size as u64,
                ),
                min_latency: Duration::from_nanos(total_time.as_nanos() as u64 / batch_size as u64),
                max_latency: Duration::from_nanos(total_time.as_nanos() as u64 / batch_size as u64),
                texts_processed: batch_size,
                total_entities_detected: total_entities,
                entities_per_second: total_entities as f64 / total_time.as_secs_f64(),
            });
        }

        results
    }

    pub fn print_performance_report(&self, results: &[PerformanceBenchmark]) {
        println!("\n🚀 PERFORMANCE BENCHMARK RESULTS");
        println!("=================================");

        // Group by model
        let mut model_results: HashMap<String, Vec<&PerformanceBenchmark>> = HashMap::new();
        for result in results {
            model_results
                .entry(result.model_name.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        for (model_name, benchmarks) in &model_results {
            println!("\n📊 {} Performance:", model_name);
            println!("{}", "─".repeat(60));

            for benchmark in benchmarks {
                println!("\n  🔍 {}", benchmark.test_name);
                println!("    📝 Texts processed: {}", benchmark.texts_processed);
                println!(
                    "    📏 Total characters: {}",
                    benchmark.total_chars_processed
                );
                println!(
                    "    ⏱️  Total time: {:.2}s",
                    benchmark.total_processing_time.as_secs_f64()
                );
                println!(
                    "    🚀 Throughput: {:.0} chars/sec",
                    benchmark.chars_per_second
                );
                println!(
                    "    ⚡ Avg latency: {:.2}ms",
                    benchmark.avg_latency_per_text.as_millis()
                );
                println!(
                    "    📊 Latency range: {:.2}ms - {:.2}ms",
                    benchmark.min_latency.as_millis(),
                    benchmark.max_latency.as_millis()
                );
                println!(
                    "    🎯 Entities found: {} ({:.1} entities/sec)",
                    benchmark.total_entities_detected, benchmark.entities_per_second
                );
            }
        }

        // Print comparative summary if multiple models
        if model_results.len() > 1 {
            self.print_comparative_summary(results);
        }
    }

    fn print_comparative_summary(&self, results: &[PerformanceBenchmark]) {
        println!("\n🏆 COMPARATIVE PERFORMANCE SUMMARY");
        println!("===================================");

        // Find throughput tests for comparison
        let throughput_tests: Vec<_> = results
            .iter()
            .filter(|r| r.test_name == "Throughput Test")
            .collect();

        if throughput_tests.len() > 1 {
            println!("\n📈 Throughput Comparison:");
            for test in &throughput_tests {
                println!(
                    "  {} - {:.0} chars/sec, {:.1} entities/sec, {:.2}ms avg latency",
                    test.model_name,
                    test.chars_per_second,
                    test.entities_per_second,
                    test.avg_latency_per_text.as_millis()
                );
            }

            // Find fastest model
            if let Some(fastest) = throughput_tests
                .iter()
                .max_by(|a, b| a.chars_per_second.partial_cmp(&b.chars_per_second).unwrap())
            {
                println!(
                    "\n🥇 Fastest Model: {} ({:.0} chars/sec)",
                    fastest.model_name, fastest.chars_per_second
                );
            }

            // Find lowest latency model
            if let Some(lowest_latency) = throughput_tests
                .iter()
                .min_by(|a, b| a.avg_latency_per_text.cmp(&b.avg_latency_per_text))
            {
                println!(
                    "⚡ Lowest Latency: {} ({:.2}ms avg)",
                    lowest_latency.model_name,
                    lowest_latency.avg_latency_per_text.as_millis()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_benchmarks() {
        let mut tester = PerformanceTester::new();

        // Add available models
        tester.add_model(
            "GLiNER Small v2.1".to_string(),
            "models/gliner/gliner_small-v2.1/tokenizer.json".to_string(),
            "models/gliner/gliner_small-v2.1/model.onnx".to_string(),
        );

        // You can add the x-small model too if available
        tester.add_model(
            "GLiNER X-Small".to_string(),
            "models/gliner/gliner-x-small/tokenizer.json".to_string(),
            "models/gliner/gliner-x-small/model.onnx".to_string(),
        );

        // Generate test data
        tester
            .with_synthetic_texts(50)
            .with_real_world_texts()
            .with_stress_test_texts();

        // Run benchmarks
        let results = tester.run_comprehensive_benchmarks();

        if !results.is_empty() {
            tester.print_performance_report(&results);

            // Basic assertion - at least one model should process some text
            let total_chars_processed: usize =
                results.iter().map(|r| r.total_chars_processed).sum();

            assert!(
                total_chars_processed > 0,
                "Should have processed some characters"
            );
        } else {
            println!(
                "⚠️  No models available for benchmarking. Run 'make download-models' to download GLiNER models."
            );
        }
    }
}
