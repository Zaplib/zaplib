# https://github.com/kostya/benchmarks/blob/1dd7deb29a813d1095e6062c25ad92bd81ce0273/json/generate_json.rb 

# frozen_string_literal: true

require 'json'

x = []

524_288.times do
  h = {
    'x' => rand * -10e-30,
    'y' => rand * 10e30,
    'z' => rand,
    'name' => "#{('a'..'z').to_a.sample(6).join} #{rand(10_000)}",
    'opts' => { '1' => [1, true] }
  }
  x << h
end

File.write(
  'data.json',
  JSON.pretty_generate('coordinates' => x, 'info' => 'some info')
)
