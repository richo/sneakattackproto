#!/usr/bin/env ruby

require 'erb'
require 'json'
require 'set'
require 'irb'

# This has to be kept in sync with the map in web. Probably do something smarter.
RALLIES = {
  "2025" => "../2025rallies.json",
  "2024" => "../2024rallies.json",
}

class RallyData
  def initialize
    @uids = Hash.new
    @rallies = Hash.new
    @all_numbers = SortedSet.new

    RALLIES.each do |year, file|
      year_data = JSON.parse(File.read(file))
      @rallies[year] = year_data
      year_data.each do |rally|
        rally["entries"].each do |entry|
          @all_numbers << entry["number"]
        end
      end
    end

    uid_list = JSON.parse(File.read('../uidsSmall.json'))
    uid_list.each do |uid|
      @uids[Integer(uid["uid"])] = uid
    end

    def ordered_rallies
      RALLIES.map do |k, _|
        [k, @rallies[k]]
      end
    end
  end

  def get_binding
    binding
  end
end

def main
  data = RallyData.new
  template = ERB.new(File.read("timecomp.html.erb"))

  rendered = template.result(data.get_binding)
  File.open("timecomp.html", "w") do |fh|
    fh.puts(rendered)
  end
end

main

