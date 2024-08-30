document.addEventListener('DOMContentLoaded', function() {
    var map = L.map('map_lpz').setView([51.34, 12.39], 12);

    let geoJsonLayer;
    let geoJsonData;
    let legend;
    let currentParty = "Gesamt"; 
    let topRange = {};
    let topRangeParty;
    const parties = ["Gesamt", "Grüne", "AfD", "BSW", "CDU", "Die Linke", "Die Partei", "FDP", , "SPD"];

    L.tileLayer('https://{s}.tile.openstreetmap.fr/hot/{z}/{x}/{y}.png', {
        maxZom: 18,
        attribution: '© OpenStreetMap contributors'
    }).addTo(map);
    
    // adds scale to map
    L.control.scale().addTo(map);

    const partyColorScales = {
      // continuous color from left to right
      "AfD": chroma.scale(['#808080', '#4A2511']),
      "BSW": chroma.scale(['#808080', '#ff944d']),
      "CDU": chroma.scale(['#808080', '#000000']),
      "Die Linke": chroma.scale(['#808080', '#ff0066']),
      "Die Partei": chroma.scale(['#808080', '#660000']),
      "FDP": chroma.scale(['#808080', '#FFD700']),
      "Grüne": chroma.scale([ '#808080', '#009900']),
      "SPD": chroma.scale(['#808080', '#ff0000'])
    };

    function getPartyColor(party) {
      const colors = {
          "AfD": "#4A2511",
          "BSW": "#FF8C00",
          "CDU": "#000000",
          "Die Linke": "#FF1493",
          "Die Partei": "#800020",
          "FDP": "#FFD700",
          "Grüne": "#006400",
          "SPD": "#FF0000"
      };
      return colors[party] || '#FFFFFF';
    }

    function getColor(feature, party) {
      if (party == "Gesamt") {
        const partyVotesArray = Object.entries(feature.properties).filter(([key, value]) => parties.includes(key) && key !== "total_votes")
        .map(([key, value]) => ({party: key, votes: value}));
        const winningParty = partyVotesArray.reduce((max, current) => current.votes > max.votes ? current : max);
        return getPartyColor(winningParty.party);
      } else {
        const scale = partyColorScales[party] || chroma.scale(['#FFEDA0', '#800026']);
        const topRangeParty = topRange[party] || 40;
        return scale(feature.properties[party] / topRangeParty).hex();
      }
    }
    
    function style(feature, party) {
      return {
        fillColor: getColor(feature, party),
        weight: 2,
        opacity: 1,
        color: 'white',
        dashArray: '3',
        fillOpacity: 0.8
      };
    }
 
    function updateLayer(party) {
      console.log('Update Layer function is called');
      let currentParty = party;
      let topRangeParty = topRange[party] || 40;
      if (!geoJsonData) {
        console.error('No GeoJSON data available');
        return;
    }
      if (geoJsonLayer) {
          map.removeLayer(geoJsonLayer);
          console.log('Layer is removed');
      }

      geoJsonLayer = L.geoJSON(geoJsonData, {
        style: function(feature) {
          // console.log('Style is build with: ', topRange);
          return style(feature, party, topRange);
        },
        onEachFeature: onEachFeature
      }).addTo(map);
      console.log('Layer is loaded with party: ', party);

      updateLegend(party, topRangeParty);
      console.log('Legend is updated');
    }

    function updateLegend(party, topRangeParty) {
      console.log('Legend is build with: ', topRangeParty);
      if (legend) {
          map.removeControl(legend);
      }

      legend = L.control({position: 'bottomright'});
      legend.onAdd = function (map) {
          var div = L.DomUtil.create('div', 'info legend');

      if (party== "Gesamt") {
        console.log('Legend is build for Gesamt');
        div.innerHTML = '<div class="legend-columns">' +
        parties.filter(p => p !== "Gesamt").map(p =>
            '<div class="legend-item">' +
            '<i style="background:' + getPartyColor(p) + '"></i>' +
            '<span>' + p + '</span>' +
            '</div>'
        ).join('') +
        '</div>';
      } else {
            var grades = Array.from({length: 9}, (_, i) => (topRangeParty * i / 8).toFixed(1));
            var height = 200;
            var width = 30;
    
            var colorBar = '<div style="width:' + width + 'px; height:' + height + 'px; background: linear-gradient(to top, ' + 
                           partyColorScales[party](0).hex() + ', ' +
                           partyColorScales[party](1).hex() + 
                           '); float:left; margin-right:10px;"></div>';
                           
                           var labels = grades.map((grade, index) => {
                var y = height - (index * height / (grades.length - 1)) - 9;
                return '<div style="position:absolute; left:' + (width + 15) + 'px; top:' + y + 'px;">' + grade + '%</div>';
            }).join('');
            
            div.innerHTML = '<div style="position:relative; height:' + height + 'px; padding-right: 40px;">' + colorBar + labels + '</div>';
          }
          return div;
        };
        legend.addTo(map);
      }
    
    var info = L.control();

    info.onAdd = function (map) {
        this._div = L.DomUtil.create('div', 'info'); // create a div with a class "info"
        this.update();
        return this._div;
    };

    info.update = function (props) {
        this._div.innerHTML = '<h4>Leipzig Stadtratswahl 2024</h4>' +  (props ?
            '<b>' + props.name_muni + '</b><br />' + 
            (currentParty === "Gesamt" ? 
              Object.entries(props)
                  .filter(([key, value]) => parties.includes(key) && key !== "Gesamt")
                  .sort((a, b) => b[1] - a[1])  // Sort by vote count, descending
                  .map(([party, percentage]) => 
                      party + ': ' + percentage + '%'
                  )
                  .join('<br />') :
              'Ergebnis: ' + currentParty + ' mit ' + props[currentParty] + '%' )
            : 'Hover über einen Wahlbezirk');
    };

    info.addTo(map);
 
    function calculateMaxValues(data) {
      parties.forEach(party => {
          topRange[party] = Math.max(...data.features.map(feature => feature.properties[party] || 0));
      });
    }

    function highlightFeature(e) {
      var layer = e.target;
  
      layer.setStyle({
          weight: 5,
          color: '#666',
          dashArray: '',
          fillOpacity: 0.7
      });
  
      layer.bringToFront();
      info.update(layer.feature.properties);
    }

    function resetHighlight(e) {
      geoJsonLayer.resetStyle(e.target);
      info.update();
    }

    function zoomToFeature(e) {
      map.fitBounds(e.target.getBounds());
    }

    function onEachFeature(feature, layer) {
      if (feature.properties && feature.properties.name_muni) {
        layer.bindTooltip(feature.properties.name_muni, {
            permanent: true,
            direction: 'center',
            className: 'polygon-label'
        });
      }

      layer.on({
          mouseover: highlightFeature,
          mouseout: resetHighlight,
          click: zoomToFeature
      });
    }

    L.Control.PartySelect = L.Control.extend({
      onAdd: function(map) {
        const select = L.DomUtil.create('select', 'party-select');
        parties.forEach(party => {
          const option = L.DomUtil.create('option', '', select);
          option.value = party;
          option.textContent = party;
        });
        L.DomEvent.on(select, 'change', function() {
          currentParty = this.value;
          updateLayer(this.value, topRange);
        });
        return select;
      }
    });
    
    // adds selection of party on topright
    const partySelect = new L.Control.PartySelect({ position: 'topright' }).addTo(map);


    console.log('Starting fetch request');
    fetch(`/data/${DATA_SOURCE}.json`)
    .then(response => {
      console.log('Received response:', response);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      return response.json();
    })
    .then(data => {
      
      if (!data || typeof data !== 'object') {
        throw new Error('Invalid GeoJSON data');
      }

      try {
          geoJsonData = data;
          console.log('Data is defined: ', geoJsonData);
          let topRange = calculateMaxValues(data);
          console.log('Maximal Percentage for each party: ', topRange);
          updateLayer(currentParty);
          console.log('Initial layer added');
       } catch (error) {
      console.error('Error creating or adding GeoJSON layer:', error);
  }});

    console.log('Map finalized');
});
