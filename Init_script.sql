insert into restaurant values
	(1, 'Напарах', 'https://yandex.ru/maps/-/CDcLmM4d', '1000-1100 ₽', '₽₽', 'Европейская, русская',
	'{
      "type": "Regular",
      "content": {
        "working_time": {
          "start_time": "08:00:00.0",
          "end_time": "22:00:00.0"
        }
      }
    }',
    100,
    '+78126400506',
    ST_MakePoint(30.299833, 60.000142));

insert into restaurant values
	(2, 'Brasserie Kriek', 'https://yandex.ru/maps/-/CDcLNL7D', '700–1500 ₽', '₽₽', 'Европейская',
	'
    {
      "type": "WithWeekends",
      "content": {
        "weekday_working_time": {
          "start_time": "11:30:00.0",
          "end_time": "23:30:00.0"
        },
        "weekend_working_time": {
          "start_time": "11:30:00.0",
          "end_time": "02:00:00.0"
        }
      }
    }',
    100,
    '+79944337352',
    ST_MakePoint(30.299903, 60.002264));

insert into manager values
    (1, null, 'Mama', 1),
    (2, null, 'Papa', 2);