default: update_rallies html/timecomp.html


update_rallies:
	curl https://sneakattackrally.com/ARACombinerThing/data/2024rallies.json > 2024rallies.json
	curl https://sneakattackrally.com/ARACombinerThing/data/2025rallies.json > 2025rallies.json

html/timecomp.html: html/timecomp.html.erb html/generate-form.rb
	cd html && ruby generate-form.rb


.PHONY: default update_rallies
