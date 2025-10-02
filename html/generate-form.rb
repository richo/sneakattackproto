#!/usr/bin/env ruby

require 'erb'
require 'irb'

# This has to be kept in sync with the map in web. Probably do something smarter.
RALLIES = {
  "2025" => "../2025rallies.json",
  "2024" => "../2024rallies.json",
}

NON_ARA_RALLIES = "../nonARArallies.json"

def main
  data = RallyData.new
  template = ERB.new(File.read("timecomp.html.erb"))

  rendered = template.result(data.get_binding)
  File.open("timecomp.html", "w") do |fh|
    fh.puts(rendered)
  end
end

main

